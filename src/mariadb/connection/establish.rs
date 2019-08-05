use super::Connection;
use crate::mariadb::{ErrPacket, OkPacket, DeContext, Deserialize, HandshakeResponsePacket, InitialHandshakePacket, Message, Capabilities, ComStmtExec, StmtExecFlag};
use bytes::{BufMut, Bytes};
use failure::{err_msg, Error};
use crate::ConnectOptions;
use std::ops::BitAnd;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
    let buf = conn.stream.next_packet().await?;
    let mut de_ctx = DeContext::new(&mut conn.context, buf);
    let initial = InitialHandshakePacket::deserialize(&mut de_ctx)?;

    de_ctx.ctx.capabilities = de_ctx.ctx.capabilities.bitand(initial.capabilities);

    let handshake: HandshakeResponsePacket = HandshakeResponsePacket {
        // Minimum client capabilities required to establish connection
        capabilities: de_ctx.ctx.capabilities,
        max_packet_size: 1024,
        extended_capabilities: Some(Capabilities::from_bits_truncate(0)),
        username: Bytes::from(options.user.unwrap_or("")),
        ..Default::default()
    };

    conn.send(handshake).await?;

    let mut ctx = DeContext::new(&mut conn.context, conn.stream.next_packet().await?);

    match ctx.decoder.peek_tag() {
        0xFF => {
            return Err(ErrPacket::deserialize(&mut ctx)?.into());
        },
        0x00 => {
            OkPacket::deserialize(&mut ctx)?;
        },
        _ => failure::bail!("Did not receive an ErrPacket nor OkPacket when one is expected"),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Error;
    use crate::mariadb::{ComStmtPrepareResp, FieldType, ResultSet, ComStmtFetch};

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
        }).await?;

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
        }).await?;

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
        }).await?;

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
        }).await?;

        conn.select_db("test").await?;

        conn.prepare("SELECT * FROM users WHERE username = ?").await?;

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
        }).await?;

        conn.select_db("test").await?;

        let mut prepared = conn.prepare("SELECT username FROM users WHERE username = ?;").await?;

        println!("{:?}", prepared);

        if let Some(param_defs) = &mut prepared.param_defs {
            param_defs[0].field_type = FieldType::MysqlTypeBlob;
        }

        let exec = ComStmtExec {
            stmt_id: -1,
            flags: StmtExecFlag::ReadOnly,
            params: Some(vec![Some(Bytes::from_static(b"daniel"))]),
            param_defs: prepared.param_defs,
        };

        println!("{:?}", ResultSet::deserialize(DeContext::with_stream(&mut conn.context, &mut conn.stream)).await?);

        let fetch = ComStmtFetch {
            stmt_id: -1,
            rows: 10,
        };

        conn.send(fetch).await?;

        let buf = conn.stream.next_packet().await?;

        println!("{:?}", buf);

//        println!("{:?}", ResultSet::deserialize(&mut DeContext::new(&mut conn.context, &buf))?);

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
