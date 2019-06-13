use super::Connection;
use crate::protocol::{
    client::{PasswordMessage, StartupMessage},
    server::Message as ServerMessage,
};
use futures::StreamExt;
use mason_core::ConnectOptions;
use std::io;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> io::Result<()> {
    // The actual connection establishing
    Ok(())
}
