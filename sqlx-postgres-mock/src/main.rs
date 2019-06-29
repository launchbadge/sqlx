#![feature(async_await)]

use futures::{io::AsyncWriteExt, TryStreamExt};
use runtime::net::TcpListener;
use std::io;

const ESTABLISH: &[u8] = b"\
    R\0\0\0\x08\0\0\0\0\
    S\0\0\0\x16application_name\0\0\
    S\0\0\0\x19client_encoding\0UTF8\0\
    S\0\0\0\x17DateStyle\0ISO, MDY\0\
    S\0\0\0\x19integer_datetimes\0on\0\
    S\0\0\0\x1bIntervalStyle\0iso_8601\0\
    S\0\0\0\x14is_superuser\0on\0\
    S\0\0\0\x19server_encoding\0UTF8\0\
    S\0\0\0\x18server_version\011.2\0\
    S\0\0\0#session_authorization\0postgres\0\
    S\0\0\0#standard_conforming_strings\0on\0\
    S\0\0\0\x11TimeZone\0UTC\0\
    K\0\0\0\x0c\0\0'\xc6\x89R\xc5+\
    Z\0\0\0\x05I";

#[runtime::main]
async fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:5433")?;
    println!("listening on {}", listener.local_addr()?);

    listener
        .incoming()
        .try_for_each_concurrent(None, async move |mut stream| {
            runtime::spawn(async move {
                stream.write_all(ESTABLISH).await?;

                Ok::<(), std::io::Error>(())
            })
            .await
        })
        .await?;

    Ok(())
}
