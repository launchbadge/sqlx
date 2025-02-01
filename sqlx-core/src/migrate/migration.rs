use std::borrow::Cow;

use sha2::{Digest, Sha384};

use crate::sql_str::{SqlSafeStr, SqlStr};

use super::MigrationType;

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub description: Cow<'static, str>,
    pub migration_type: MigrationType,
    pub sql: SqlStr,
    pub checksum: Cow<'static, [u8]>,
    pub no_tx: bool,
}

impl Migration {
    pub fn new(
        version: i64,
        description: Cow<'static, str>,
        migration_type: MigrationType,
        sql: impl SqlSafeStr,
        no_tx: bool,
    ) -> Self {
        let sql = sql.into_sql_str();
        let checksum = Cow::Owned(Vec::from(
            Sha384::digest(sql.as_str().as_bytes()).as_slice(),
        ));

        Migration {
            version,
            description,
            migration_type,
            sql,
            checksum,
            no_tx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub checksum: Cow<'static, [u8]>,
}
