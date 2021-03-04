use std::borrow::Cow;

use bytes::buf::Chain;
use bytes::Bytes;
use sqlx_core::{Error, Result};

/// Dialog authentication implementation
///
/// https://mariadb.com/kb/en/authentication-plugin-pam/#dialog
///
#[derive(Debug)]
pub(crate) struct DialogAuthPlugin;

impl super::AuthPlugin for DialogAuthPlugin {
    fn name(&self) -> &'static str {
        "dialog"
    }

    fn invoke(&self, _nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8> {
        password.as_bytes().to_vec()
    }

    fn handle(
        &self,
        _command: u8,
        _data: Bytes,
        _nonce: &Chain<Bytes, Bytes>,
        _password: &str,
    ) -> Result<Option<Vec<u8>>> {
        Err(Error::ConnectOptions {
            message: Cow::Borrowed("interactive dialog authentication is currently not supported"),
            source: None,
        })
    }
}
