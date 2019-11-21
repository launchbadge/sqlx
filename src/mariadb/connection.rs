use super::establish;
use crate::{
    connection::RawConnection,
    describe::{Describe, ResultField},
    error::DatabaseError,
    io::{Buf, BufMut, BufStream},
    mariadb::{
        protocol::{
            Capabilities, ColumnCountPacket, ColumnDefinitionPacket, ComPing, ComQuit,
            ComStmtExecute, ComStmtPrepare, ComStmtPrepareOk, Encode, EofPacket, ErrPacket,
            OkPacket, ResultRow, StmtExecFlag,
        },
        MariaDb, MariaDbQueryParameters, MariaDbRow,
    },
    Backend, Error, Result,
};
use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::{self, StreamExt};
use std::{
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
};
use async_std::net::TcpStream;
use url::Url;

pub struct MariaDbRawConnection {
    pub(crate) stream: BufStream<TcpStream>,
    pub(crate) rbuf: Vec<u8>,
    pub(crate) capabilities: Capabilities,
    next_seq_no: u8,
}

impl MariaDbRawConnection {
    async fn establish(url: &str) -> Result<Self> {
        // TODO: Handle errors
        let url = Url::parse(url).unwrap();

        let host = url.host_str().unwrap_or("127.0.0.1");
        let port = url.port().unwrap_or(3306);

        // TODO: handle errors
        let host: IpAddr = host.parse().unwrap();
        let addr: SocketAddr = (host, port).into();

        let stream = TcpStream::connect(&addr).await?;

        let mut conn = Self {
            stream: BufStream::new(stream),
            rbuf: Vec::with_capacity(8 * 1024),
            capabilities: Capabilities::empty(),
            next_seq_no: 0,
        };

        establish::establish(&mut conn, &url).await?;

        Ok(conn)
    }

    pub async fn close(mut self) -> Result<()> {
        // Send the quit command

        self.start_sequence();
        self.write(ComQuit);

        self.stream.flush().await?;

        Ok(())
    }

    pub async fn ping(&mut self) -> Result<()> {
        // Send the ping command and wait for (and drop) an OK packet

        self.start_sequence();
        self.write(ComPing);

        self.stream.flush().await?;

        let _ = self.receive_ok_or_err().await?;

        Ok(())
    }

    pub(crate) async fn receive(&mut self) -> Result<&[u8]> {
        Ok(self
            .try_receive()
            .await?
            .ok_or(Error::Io(io::ErrorKind::UnexpectedEof.into()))?)
    }

    async fn try_receive(&mut self) -> Result<Option<&[u8]>> {
        // Read the packet header which contains the length and the sequence number
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet
        let mut header = ret_if_none!(self.stream.peek(4).await?);
        let len = header.get_u24::<LittleEndian>()? as usize;
        self.next_seq_no = header.get_u8()? + 1;
        self.stream.consume(4);

        // Read the packet body and copy it into our internal buf
        // We must have a separate buffer around the stream as we can't operate directly
        // on bytes returend from the stream. We have compression, split, etc. to
        // unpack.
        let body = ret_if_none!(self.stream.peek(len).await?);
        self.rbuf.clear();
        self.rbuf.extend_from_slice(body);
        self.stream.consume(len);

        Ok(Some(&self.rbuf[..len]))
    }

    fn start_sequence(&mut self) {
        // At the start of a command sequence we reset our understanding
        // of [next_seq_no]. In a sequence our initial command must be 0, followed
        // by the server response that is 1, then our response to that response (if any),
        // would be 2
        self.next_seq_no = 0;
    }

    pub(crate) fn write<T: Encode>(&mut self, packet: T) {
        let buf = self.stream.buffer_mut();

        // Allocate room for the header that we write after the packet;
        // so, we can get an accurate and cheap measure of packet length

        let header_offset = buf.len();
        buf.advance(4);

        packet.encode(buf, self.capabilities);

        // Determine length of encoded packet
        // and write to allocated header

        let len = buf.len() - header_offset - 4;
        let mut header = &mut buf[header_offset..];

        LittleEndian::write_u32(&mut header, len as u32); // len

        // Take the last sequence number received, if any, and increment by 1
        // If there was no sequence number, we only increment if we split packets
        header[3] = self.next_seq_no;
        self.next_seq_no += 1;
    }

    // Decode an OK packet or bubble an ERR packet as an error
    // to terminate immediately
    pub(crate) async fn receive_ok_or_err(&mut self) -> Result<OkPacket> {
        let capabilities = self.capabilities;
        let mut buf = self.receive().await?;
        Ok(match buf[0] {
            0xfe | 0x00 => OkPacket::decode(buf, capabilities)?,

            0xff => {
                let err = ErrPacket::decode(buf)?;

                // TODO: Bubble as Error::Database
                //                panic!("received db err = {:?}", err);
                return Err(
                    io::Error::new(io::ErrorKind::InvalidInput, format!("{:?}", err)).into(),
                );
            }

            id => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "unexpected packet identifier 0x{:X?} when expecting 0xFE (OK) or 0xFF \
                         (ERR)",
                        id
                    ),
                )
                .into());
            }
        })
    }

    async fn check_eof(&mut self) -> Result<()> {
        if !self
            .capabilities
            .contains(Capabilities::CLIENT_DEPRECATE_EOF)
        {
            let _ = EofPacket::decode(self.receive().await?)?;
        }

        Ok(())
    }

    async fn send_prepare<'c>(&'c mut self, statement: &'c str) -> Result<ComStmtPrepareOk> {
        self.stream.flush().await?;

        self.start_sequence();
        self.write(ComStmtPrepare { statement });

        self.stream.flush().await?;

        // COM_STMT_PREPARE returns COM_STMT_PREPARE_OK (0x00) or ERR (0xFF)
        let packet = self.receive().await?;

        if packet[0] == 0xFF {
            return ErrPacket::decode(packet)?.expect_error();
        }

        ComStmtPrepareOk::decode(packet).map_err(Into::into)
    }

    async fn execute(&mut self, statement_id: u32, params: MariaDbQueryParameters) -> Result<u64> {
        // TODO: EXECUTE(READ_ONLY) => FETCH instead of EXECUTE(NO)

        // SEND ================
        self.start_sequence();
        self.write(ComStmtExecute {
            statement_id,
            params: &[],
            null: &[],
            flags: StmtExecFlag::NO_CURSOR,
            param_types: &[],
        });
        self.stream.flush().await?;
        // =====================

        // Row Counter, used later
        let mut rows = 0u64;
        let capabilities = self.capabilities;
        let has_eof = capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF);

        let packet = self.receive().await?;
        if packet[0] == 0x00 {
            let _ok = OkPacket::decode(packet, capabilities)?;
        } else if packet[0] == 0xFF {
            let err = ErrPacket::decode(packet)?;
            panic!("received db err = {:?}", err);
        } else {
            // A Resultset starts with a [ColumnCountPacket] which is a single field that encodes
            // how many columns we can expect when fetching rows from this statement
            let column_count: u64 = ColumnCountPacket::decode(packet)?.columns;

            // Next we have a [ColumnDefinitionPacket] which verbosely explains each minute
            // detail about the column in question including table, aliasing, and type
            // TODO: This information was *already* returned by PREPARE .., is there a way to suppress generation
            let mut columns = vec![];
            for _ in 0..column_count {
                columns.push(ColumnDefinitionPacket::decode(self.receive().await?)?);
            }

            // When (legacy) EOFs are enabled, the fixed number column definitions are further terminated by
            // an EOF packet
            if !has_eof {
                let _eof = EofPacket::decode(self.receive().await?)?;
            }

            // For each row in the result set we will receive a ResultRow packet.
            // We may receive an [OkPacket], [EofPacket], or [ErrPacket] (depending on if EOFs are enabled) to finalize the iteration.
            loop {
                let packet = self.receive().await?;
                if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                    // NOTE: It's possible for a ResultRow to start with 0xFE (which would normally signify end-of-rows)
                    //       but it's not possible for an Ok/Eof to be larger than 0xFF_FF_FF.
                    if !has_eof {
                        let _eof = EofPacket::decode(packet)?;
                    } else {
                        let _ok = OkPacket::decode(packet, capabilities)?;
                    }

                    break;
                } else if packet[0] == 0xFF {
                    let err = ErrPacket::decode(packet)?;
                    panic!("received db err = {:?}", err);
                } else {
                    // Ignore result rows; exec only returns number of affected rows;
                    let _ = ResultRow::decode(packet, &columns)?;

                    // For every row we decode we increment counter
                    rows = rows + 1;
                }
            }
        }

        Ok(rows)
    }
}

enum ExecResult {
    NoRows(OkPacket),
    Rows(Vec<ColumnDefinitionPacket>),
}

#[async_trait]
impl RawConnection for MariaDbRawConnection {
    type Backend = MariaDb;

    async fn establish(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        MariaDbRawConnection::establish(url).await
    }

    async fn close(mut self) -> crate::Result<()> {
        self.close().await
    }

    async fn ping(&mut self) -> crate::Result<()> {
        self.ping().await
    }

    async fn execute(&mut self, query: &str, params: MariaDbQueryParameters) -> crate::Result<u64> {
        // Write prepare statement to buffer
        self.start_sequence();
        let prepare_ok = self.send_prepare(query).await?;

        let affected = self.execute(prepare_ok.statement_id, params).await?;

        Ok(affected)
    }

    fn fetch(
        &mut self,
        query: &str,
        params: MariaDbQueryParameters,
    ) -> BoxStream<'_, Result<MariaDbRow>> {
        unimplemented!();
    }

    async fn fetch_optional(
        &mut self,
        query: &str,
        params: MariaDbQueryParameters,
    ) -> crate::Result<Option<<Self::Backend as Backend>::Row>> {
        unimplemented!();
    }

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<MariaDb>> {
        let prepare_ok = self.send_prepare(query).await?;

        let mut param_types = Vec::with_capacity(prepare_ok.params as usize);

        for _ in 0..prepare_ok.params {
            let param = ColumnDefinitionPacket::decode(self.receive().await?)?;
            param_types.push(param.field_type.0);
        }

        self.check_eof().await?;

        let mut columns = Vec::with_capacity(prepare_ok.columns as usize);

        for _ in 0..prepare_ok.columns {
            let column = ColumnDefinitionPacket::decode(self.receive().await?)?;
            columns.push(ResultField {
                name: column.column_alias.or(column.column),
                table_id: column.table_alias.or(column.table),
                type_id: column.field_type.0,
            })
        }

        self.check_eof().await?;

        Ok(Describe {
            param_types,
            result_fields: columns,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{query::QueryParameters, Error, Pool};

    #[async_std::test]
    async fn it_can_connect() -> Result<()> {
        MariaDbRawConnection::establish("mariadb://root@127.0.0.1:3306/test").await?;
        Ok(())
    }

    #[async_std::test]
    async fn it_fails_to_connect_with_bad_username() -> Result<()> {
        match MariaDbRawConnection::establish("mariadb://roote@127.0.0.1:3306/test").await {
            Ok(_) => panic!("Somehow connected to database with incorrect username"),
            Err(_) => Ok(()),
        }
    }

    #[async_std::test]
    async fn it_can_ping() -> Result<()> {
        let mut conn =
            MariaDbRawConnection::establish("mariadb://root@127.0.0.1:3306/test").await?;
        conn.ping().await?;
        Ok(())
    }

    #[async_std::test]
    async fn it_can_describe() -> Result<()> {
        let mut conn =
            MariaDbRawConnection::establish("mysql://sqlx_user@127.0.0.1:3306/sqlx_test").await?;
        let describe = conn.describe("SELECT id from accounts where id = ?").await?;

        dbg!(describe);

        Ok(())
    }

    #[async_std::test]
    async fn it_can_create_mariadb_pool() -> Result<()> {
        let pool: Pool<MariaDb> = Pool::new("mariadb://root@127.0.0.1:3306/test").await?;
        Ok(())
    }
}
