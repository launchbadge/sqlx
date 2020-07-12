use std::borrow::Cow;

use sha2::{Digest, Sha384};

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub description: Cow<'static, str>,
    pub sql: Cow<'static, str>,
    pub checksum: Cow<'static, [u8]>,
}

impl Migration {
    pub fn new(version: i64, description: Cow<'static, str>, sql: Cow<'static, str>) -> Self {
        let checksum = Cow::Owned(Vec::from(Sha384::digest(sql.as_bytes()).as_slice()));

        Migration {
            version,
            description,
            sql,
            checksum,
        }
    }
}
