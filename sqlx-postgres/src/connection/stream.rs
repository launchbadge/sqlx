use std::str::FromStr;

use crate::connection::tls::MaybeUpgradeTls;
use crate::error::Error;
use crate::net::{self, BufferedSocket, Socket};
use crate::PgConnectOptions;

// the stream is a separate type from the connection to uphold the invariant where an instantiated
// [PgConnection] is a **valid** connection to postgres

// when a new connection is asked for, we work directly on the [PgStream] type until the
// connection is fully established

// in other words, `self` in any PgConnection method is a live connection to postgres that
// is fully prepared to receive queries

pub struct PgStream {
    // A trait object is okay here as the buffering amortizes the overhead of both the dynamic
    // function call as well as the syscall.
    inner: BufferedSocket<Box<dyn Socket>>,
}

impl PgStream {
    pub fn into_inner(self) -> BufferedSocket<Box<dyn Socket>> {
        self.inner
    }

    pub(super) async fn connect(options: &PgConnectOptions) -> Result<Self, Error> {
        let socket_result = match options.fetch_socket() {
            Some(ref path) => net::connect_uds(path, MaybeUpgradeTls(options)).await?,
            None => net::connect_tcp(&options.host, options.port, MaybeUpgradeTls(options)).await?,
        };

        let socket = socket_result?;

        Ok(Self {
            inner: BufferedSocket::new(socket),
        })
    }
}

// reference:
// https://github.com/postgres/postgres/blob/6feebcb6b44631c3dc435e971bd80c2dd218a5ab/src/interfaces/libpq/fe-exec.c#L1030-L1065
pub fn parse_server_version(s: String) -> Option<u32> {
    let mut parts = Vec::<u32>::with_capacity(3);

    let mut from = 0;
    let mut chs = s.char_indices().peekable();
    while let Some((i, ch)) = chs.next() {
        match ch {
            '.' => {
                if let Ok(num) = u32::from_str(&s[from..i]) {
                    parts.push(num);
                    from = i + 1;
                } else {
                    break;
                }
            }
            _ if ch.is_ascii_digit() => {
                if chs.peek().is_none() {
                    if let Ok(num) = u32::from_str(&s[from..]) {
                        parts.push(num);
                    }
                    break;
                }
            }
            _ => {
                if let Ok(num) = u32::from_str(&s[from..i]) {
                    parts.push(num);
                }
                break;
            }
        };
    }

    let version_num = match parts.as_slice() {
        [major, minor, rev] => (100 * major + minor) * 100 + rev,
        [major, minor] if *major >= 10 => 100 * 100 * major + minor,
        [major, minor] => (100 * major + minor) * 100,
        [major] => 100 * 100 * major,
        _ => return None,
    };

    Some(version_num)
}

#[cfg(test)]
mod tests {
    use super::parse_server_version;

    #[test]
    fn test_parse_server_version_num() {
        // old style
        assert_eq!(parse_server_version("9.6.1"), Some(90601));
        // new style
        assert_eq!(parse_server_version("10.1"), Some(100001));
        // old style without minor version
        assert_eq!(parse_server_version("9.6devel"), Some(90600));
        // new style without minor version, e.g.  */
        assert_eq!(parse_server_version("10devel"), Some(100000));
        assert_eq!(parse_server_version("13devel87"), Some(130000));
        // unknown
        assert_eq!(parse_server_version("unknown"), None);
    }
}
