use crate::mariadb::MariaDbRawConnection;
use std::io;

pub async fn execute(conn: &mut MariaDbRawConnection) -> io::Result<u64> {
    conn.flush().await?;

    let mut rows: u64 = 0;
    while let Some(message) = conn.receive().await? {}
}
