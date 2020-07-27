use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub description: Cow<'static, str>,
    pub sql: Cow<'static, str>,
    pub checksum: Cow<'static, [u8]>,
}
