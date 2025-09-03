use sha2::{Digest, Sha384};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::sql_str::{SqlSafeStr, SqlStr};

use super::{MigrateError, MigrationType};

const ENABLE_SUBSTITUTION: &str = "-- enable-substitution";
const DISABLE_SUBSTITUTION: &str = "-- disable-substitution";

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
        sql: SqlStr,
        no_tx: bool,
    ) -> Self {
        let checksum = checksum(sql.as_str());

        Self::with_checksum(
            version,
            description,
            migration_type,
            sql,
            checksum.into(),
            no_tx,
        )
    }

    pub(crate) fn with_checksum(
        version: i64,
        description: Cow<'static, str>,
        migration_type: MigrationType,
        sql: SqlStr,
        checksum: Cow<'static, [u8]>,
        no_tx: bool,
    ) -> Self {
        Migration {
            version,
            description,
            migration_type,
            sql,
            checksum,
            no_tx,
        }
    }

    fn name(&self) -> String {
        let description = self.description.replace(' ', "_");
        match self.migration_type {
            MigrationType::Simple => {
                format!("{}_{}", self.version, description)
            }
            MigrationType::ReversibleUp => {
                format!("{}_{}.{}", self.version, description, "up")
            }
            MigrationType::ReversibleDown => {
                format!("{}_{}.{}", self.version, description, "down")
            }
        }
    }

    pub fn process_parameters(
        &self,
        params: &HashMap<String, String>,
    ) -> Result<Self, MigrateError> {
        let Migration {
            version,
            description,
            migration_type,
            sql,
            checksum,
            no_tx,
        } = self;

        let mut new_sql = String::with_capacity(sql.as_str().len());
        let mut substitution_enabled = false;

        for (i, line) in sql.as_str().lines().enumerate() {
            if i != 0 {
                new_sql.push('\n')
            }
            let trimmed_line = line.trim();
            if trimmed_line == ENABLE_SUBSTITUTION {
                substitution_enabled = true;
                new_sql.push_str(line);
                continue;
            } else if trimmed_line == DISABLE_SUBSTITUTION {
                new_sql.push_str(line);
                substitution_enabled = false;
                continue;
            }

            if substitution_enabled {
                let substituted_line = subst::substitute(line, params).map_err(|e| match e {
                    subst::Error::NoSuchVariable(subst::error::NoSuchVariable {
                        position,
                        name,
                    }) => MigrateError::MissingParameter(self.name(), name, i + 1, position),
                    _ => MigrateError::InvalidParameterSyntax(e.to_string()),
                })?;
                new_sql.push_str(&substituted_line);
            } else {
                new_sql.push_str(line);
            }
        }

        Ok(Migration {
            version: *version,
            description: description.clone(),
            migration_type: *migration_type,
            sql: crate::sql_str::AssertSqlSafe(new_sql).into_sql_str(),
            checksum: checksum.clone(),
            no_tx: *no_tx,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub checksum: Cow<'static, [u8]>,
}

pub fn checksum(sql: &str) -> Vec<u8> {
    Vec::from(Sha384::digest(sql).as_slice())
}

pub fn checksum_fragments<'a>(fragments: impl Iterator<Item = &'a str>) -> Vec<u8> {
    let mut digest = Sha384::new();

    for fragment in fragments {
        digest.update(fragment);
    }

    digest.finalize().to_vec()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_migration_process_parameters_with_substitution() -> Result<(), MigrateError> {
        const CREATE_USER: &str = r#"
            -- enable-substitution
            CREATE USER '${substitution_test_user}';
            -- disable-substitution
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
            -- enable-substitution
            DROP USER '${substitution_test_user}';
            -- disable-substitution
        "#;
        const EXPECTED_RESULT: &str = r#"
            -- enable-substitution
            CREATE USER 'my_user';
            -- disable-substitution
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
            -- enable-substitution
            DROP USER 'my_user';
            -- disable-substitution
        "#;

        let migration = Migration::new(
            1,
            Cow::Owned("test a simple parameter substitution".to_string()),
            crate::migrate::MigrationType::Simple,
            crate::sql_str::AssertSqlSafe(CREATE_USER.to_string()).into_sql_str(),
            true,
        );
        let result = migration.process_parameters(&HashMap::from([(
            String::from("substitution_test_user"),
            String::from("my_user"),
        )]))?;
        assert_eq!(result.sql, EXPECTED_RESULT);
        Ok(())
    }

    #[test]
    fn test_migration_process_parameters_no_substitution() -> Result<(), MigrateError> {
        const CREATE_TABLE: &str = r#"
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
        "#;
        let migration = Migration::new(
            1,
            std::borrow::Cow::Owned("test a simple parameter substitution".to_string()),
            crate::migrate::MigrationType::Simple,
            crate::sql_str::AssertSqlSafe(CREATE_TABLE.to_string()).into_sql_str(),
            true,
        );
        let result = migration.process_parameters(&HashMap::from([(
            String::from("substitution_test_user"),
            String::from("my_user"),
        )]))?;
        assert_eq!(result.sql, CREATE_TABLE);
        Ok(())
    }

    #[test]
    fn test_migration_process_parameters_missing_key() -> Result<(), MigrateError> {
        const CREATE_TABLE: &str = r#"
            -- enable-substitution
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT,
                field ${TEST_MISSING_KEY}
            );
            -- disable-substitution

        "#;
        let migration = Migration::new(
            1,
            Cow::Owned("test a simple parameter substitution".to_string()),
            crate::migrate::MigrationType::Simple,
            crate::sql_str::AssertSqlSafe(CREATE_TABLE.to_string()).into_sql_str(),
            true,
        );
        let Err(MigrateError::MissingParameter(..)) =
            migration.process_parameters(&HashMap::with_capacity(0))
        else {
            panic!("Missing env var not caught in process parameters missing env var")
        };
        Ok(())
    }

    #[test]
    fn test_migration_process_parameters_missing_key_with_default_value() -> Result<(), MigrateError>
    {
        const CREATE_TABLE: &str = r#"
            -- enable-substitution
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT,
                field ${TEST_MISSING_KEY:TEXT}
            );
            -- disable-substitution
        "#;
        const EXPECTED_CREATE_TABLE: &str = r#"
            -- enable-substitution
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT,
                field TEXT
            );
            -- disable-substitution
        "#;
        let migration = Migration::new(
            1,
            Cow::Owned("test a simple parameter substitution".to_string()),
            crate::migrate::MigrationType::Simple,
            crate::sql_str::AssertSqlSafe(CREATE_TABLE.to_string()).into_sql_str(),
            true,
        );
        let result = migration.process_parameters(&HashMap::with_capacity(0))?;
        assert_eq!(result.sql, EXPECTED_CREATE_TABLE);
        Ok(())
    }

    #[test]
    fn fragments_checksum_equals_full_checksum() {
        // Copied from `examples/postgres/axum-social-with-tests/migrations/3_comment.sql`
        let sql = "\
            \u{FEFF}create table comment (\r\n\
                \tcomment_id uuid primary key default gen_random_uuid(),\r\n\
                \tpost_id uuid not null references post(post_id),\r\n\
                \tuser_id uuid not null references \"user\"(user_id),\r\n\
                \tcontent text not null,\r\n\
                \tcreated_at timestamptz not null default now()\r\n\
            );\r\n\
            \r\n\
            create index on comment(post_id, created_at);\r\n\
        ";

        // Should yield a string for each character
        let fragments_checksum = checksum_fragments(sql.split(""));
        let full_checksum = checksum(sql);

        assert_eq!(fragments_checksum, full_checksum);
    }
}
