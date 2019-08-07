use super::Connection;
use crate::{
    mariadb::{
        Capabilities, ComStmtExec, DeContext, Decode, EofPacket, ErrPacket,
        HandshakeResponsePacket, InitialHandshakePacket, OkPacket, ProtocolType, StmtExecFlag,
    },
    options::ConnectOptions,
};
use bytes::Bytes;
use failure::{err_msg, Error};
use std::ops::BitAnd;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
    let buf = conn.stream.next_packet().await?;
    let mut de_ctx = DeContext::new(&mut conn.context, buf);
    let initial = InitialHandshakePacket::decode(&mut de_ctx)?;

    de_ctx.ctx.capabilities = de_ctx.ctx.capabilities.bitand(initial.capabilities);

    let handshake: HandshakeResponsePacket = HandshakeResponsePacket {
        // Minimum client capabilities required to establish connection
        capabilities: de_ctx.ctx.capabilities,
        max_packet_size: 1024,
        extended_capabilities: Some(Capabilities::from_bits_truncate(0)),
        username: options.user.unwrap_or(""),
        ..Default::default()
    };

    conn.send(handshake).await?;

    let mut ctx = DeContext::new(&mut conn.context, conn.stream.next_packet().await?);

    match ctx.decoder.peek_tag() {
        0xFF => {
            return Err(ErrPacket::decode(&mut ctx)?.into());
        }
        0x00 => {
            OkPacket::decode(&mut ctx)?;
        }
        _ => failure::bail!("Did not receive an ErrPacket nor OkPacket when one is expected"),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mariadb::{ComStmtFetch, ComStmtPrepareResp, FieldType, ResultSet};
    use failure::Error;

    #[runtime::test]
    async fn it_can_connect() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_can_ping() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.ping().await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_can_select_db() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.select_db("test").await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_can_query() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.select_db("test").await?;

        conn.query("SELECT * FROM users").await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_can_prepare() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.select_db("test").await?;

        conn.prepare("SELECT * FROM users WHERE username = ?")
            .await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_can_execute_prepared() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.select_db("test").await?;

        let mut prepared = conn
            .prepare("SELECT id FROM users WHERE username=?")
            .await?;

        let exec = ComStmtExec {
            stmt_id: prepared.ok.stmt_id,
            flags: StmtExecFlag::NoCursor,
            params: Some(vec![Some(Bytes::from_static(b"josh"))]),
            param_defs: prepared.param_defs,
        };

        conn.send(exec).await?;

        let mut ctx = DeContext::with_stream(&mut conn.context, &mut conn.stream);
        ctx.next_packet().await?;
        ctx.columns = Some(prepared.ok.columns as u64);
        ctx.column_defs = prepared.res_columns;

        println!("{:?}", ctx.columns);
        println!("{:?}", ctx.column_defs);

        match ctx.decoder.peek_tag() {
            0xFF => {
                ErrPacket::decode(&mut ctx)?;
            }
            0x00 => {
                OkPacket::decode(&mut ctx)?;
            }
            _ => {
                ResultSet::deserialize(ctx, ProtocolType::Binary).await?;
            }
        }

        Ok(())
    }

    #[runtime::test]
    async fn it_does_not_connect() -> Result<(), Error> {
        match Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("roote"),
            database: None,
            password: None,
        })
        .await
        {
            Ok(_) => Err(err_msg("Bad username still worked?")),
            Err(_) => Ok(()),
        }
    }
}
