use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Migration {
    pub(crate) version: i64,
    pub(crate) description: Cow<'static, str>,
    pub(crate) sql: Cow<'static, str>,
    pub(crate) checksum: Cow<'static, [u8]>,
}

impl Migration {
    pub fn version(&self) -> i64 {
        self.version
    }

    pub fn description(&self) -> &str {
        &*self.description
    }
}
