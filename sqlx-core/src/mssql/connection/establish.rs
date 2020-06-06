use crate::error::Error;
use crate::io::Decode;
use crate::mssql::connection::stream::MsSqlStream;
use crate::mssql::protocol::login::Login7;
use crate::mssql::protocol::login_ack::LoginAck;
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::pre_login::{Encrypt, PreLogin, Version};
use crate::mssql::{MsSqlConnectOptions, MsSqlConnection};

impl MsSqlConnection {
    pub(crate) async fn establish(options: &MsSqlConnectOptions) -> Result<Self, Error> {
        let mut stream: MsSqlStream = MsSqlStream::connect(options).await?;

        // Send PRELOGIN to set up the context for login. The server should immediately
        // respond with a PRELOGIN message of its own.

        // TODO: Encryption
        // TODO: Send the version of SQLx over

        stream.write_packet(
            PacketType::PreLogin,
            PreLogin {
                version: Version::default(),
                encryption: Encrypt::NOT_SUPPORTED,

                ..Default::default()
            },
        );

        stream.flush().await?;

        let (_, packet) = stream.recv_packet().await?;
        let _ = PreLogin::decode(packet)?;

        // LOGIN7 defines the authentication rules for use between client and server

        stream.write_packet(
            PacketType::Tds7Login,
            Login7 {
                // FIXME: use a version constant
                version: 0x74000004, // SQL Server 2012 - SQL Server 2019
                client_program_version: 0,
                client_pid: 0,
                packet_size: 4096,
                hostname: "",
                username: &options.username,
                password: options.password.as_deref().unwrap_or_default(),
                app_name: "",
                server_name: "",
                client_interface_name: "",
                language: "",
                // FIXME: connect this to options.database
                database: "",
                client_id: [0; 6],
            },
        );

        stream.flush().await?;

        loop {
            // NOTE: we should receive an [Error] message if something goes wrong, otherwise,
            //       all messages are mostly informational (ENVCHANGE, INFO, LOGINACK)

            match stream.recv_message().await? {
                Message::LoginAck(_) => {
                    // indicates that the login was successful
                    // no action is needed, we are just going to keep waiting till we hit <Done>
                }

                Message::Done(_) => {
                    break;
                }

                _ => {}
            }
        }

        Ok(Self { stream })
    }
}
