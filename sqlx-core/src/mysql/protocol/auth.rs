use std::str::FromStr;

use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum AuthPlugin {
    MySqlNativePassword,
    CachingSha2Password,
    Sha256Password,
    Dialog,
}

impl AuthPlugin {
    pub(crate) fn name(self) -> &'static str {
        match self {
            AuthPlugin::MySqlNativePassword => "mysql_native_password",
            AuthPlugin::CachingSha2Password => "caching_sha2_password",
            AuthPlugin::Sha256Password => "sha256_password",
            AuthPlugin::Dialog => "dialog",
        }
    }

    // See: https://github.com/mysql/mysql-server/blob/ea7d2e2d16ac03afdd9cb72a972a95981107bf51/sql/auth/sha2_password.cc#L942
    pub(crate) fn auth_switch_request_data_length(self) -> usize {
        use AuthPlugin::*;
        match self {
            MySqlNativePassword | CachingSha2Password | Sha256Password => 21,
            Dialog => 0,
        }
    }
}

impl FromStr for AuthPlugin {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mysql_native_password" => Ok(AuthPlugin::MySqlNativePassword),
            "caching_sha2_password" => Ok(AuthPlugin::CachingSha2Password),
            "sha256_password" => Ok(AuthPlugin::Sha256Password),
            "dialog" => Ok(AuthPlugin::Dialog),

            _ => Err(err_protocol!("unknown authentication plugin: {}", s)),
        }
    }
}
