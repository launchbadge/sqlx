use super::Connection;
use crate::mariadb::{DeContext, Deserialize, HandshakeResponsePacket, InitialHandshakePacket, Message, Capabilities, ComStmtExec, StmtExecFlag};
use bytes::{BufMut, Bytes};
use failure::{err_msg, Error};
use crate::ConnectOptions;
use std::ops::BitAnd;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
    let buf = &conn.stream.next_bytes().await?;
    let mut de_ctx = DeContext::new(&mut conn.context, &buf);
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

    match conn.next().await? {
        Some(Message::OkPacket(message)) => {
            conn.context.seq_no = message.seq_no;
            Ok(())
        }

        Some(Message::ErrPacket(message)) => Err(err_msg(format!("{:?}", message))),

        Some(message) => {
            panic!("Did not receive OkPacket nor ErrPacket. Received: {:?}", message);
        }

        None => {
            panic!("Did not receive packet");
        }
    }
}

#[cfg(test)]
mod test {
//    use super::*;
//    use failure::Error;
//    use crate::mariadb::{ComStmtPrepareResp, FieldType, ResultSet, ComStmtFetch};
//
//    #[runtime::test]
//    async fn it_can_connect() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        })
//        .await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_can_ping() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        }).await?;
//
//        conn.ping().await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_can_select_db() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        }).await?;
//
//        conn.select_db("test").await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_can_query() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        }).await?;
//
//        conn.select_db("test").await?;
//
//        conn.query("SELECT * FROM users").await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_can_prepare() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        }).await?;
//
//        conn.select_db("test").await?;
//
//        conn.prepare("SELECT * FROM users WHERE username = ?").await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_can_execute_prepared() -> Result<(), Error> {
//        let mut conn = Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("root"),
//            database: None,
//            password: None,
//        }).await?;
//
//        conn.select_db("test").await?;
//
//        let mut prepared = conn.prepare("SELECT * FROM users WHERE username = ?;").await?;
//
//        println!("{:?}", prepared);
//
//        if let Some(param_defs) = &mut prepared.param_defs {
//            param_defs[0].field_type = FieldType::MysqlTypeBlob;
//        }
//
//        let exec = ComStmtExec {
//            stmt_id: prepared.ok.stmt_id,
//            flags: StmtExecFlag::CursorForUpdate,
//            params: Some(vec![Some(Bytes::from_static(b"'daniel'"))]),
//            param_defs: prepared.param_defs,
//        };
//
//
//        conn.send(exec).await?;
//
//        let buf = conn.stream.next_bytes().await?;
//
//        println!("{:?}", buf);
//
//        println!("{:?}", ResultSet::deserialize(&mut DeContext::new(&mut conn.context, &buf))?.rows);
//
//        let fetch = ComStmtFetch {
//            stmt_id: prepared.ok.stmt_id,
//            rows: 1,
//        };
//
//        conn.send(exec).await?;
//
//        Ok(())
//    }
//
//    #[runtime::test]
//    async fn it_does_not_connect() -> Result<(), Error> {
//        match Connection::establish(ConnectOptions {
//            host: "127.0.0.1",
//            port: 3306,
//            user: Some("roote"),
//            database: None,
//            password: None,
//        })
//        .await
//        {
//            Ok(_) => Err(err_msg("Bad username still worked?")),
//            Err(_) => Ok(()),
//        }
//    }
}
