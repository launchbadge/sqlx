use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;
use sha2::{Digest, Sha384};

use super::{MigrateError, MigrationType};

static ENV_SUB_REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub description: Cow<'static, str>,
    pub migration_type: MigrationType,
    pub sql: Cow<'static, str>,
    pub checksum: Cow<'static, [u8]>,
    pub no_tx: bool,
}

impl Migration {
    pub fn new(
        version: i64,
        description: Cow<'static, str>,
        migration_type: MigrationType,
        sql: Cow<'static, str>,
        no_tx: bool,
    ) -> Self {
        let checksum = Cow::Owned(Vec::from(Sha384::digest(sql.as_bytes()).as_slice()));
        Migration {
            version,
            description,
            migration_type,
            sql,
            checksum,
            no_tx,
        }
    }

    pub fn process_parameters(&self) -> Result<Self, MigrateError> {
        let Migration {
            version,
            description,
            migration_type,
            sql,
            checksum,
            no_tx,
        } = self;
        let re = ENV_SUB_REGEX.get_or_init(|| {
            Regex::new(r"--\s?+\+sqlx envsub on((.|\n|\r)*?)--\s?+\+sqlx envsub off").unwrap()
        });
        let mut new_sql = String::with_capacity(sql.len());
        let mut last_match = 0;
        //Use re.captures_iter over replace_all for fallibility
        for cap in re.captures_iter(sql) {
            let m = cap.get(1).unwrap();
            new_sql.push_str(&sql[last_match..m.start()]);
            let replacement = subst::substitute(&cap[1], &subst::Env)
                .map_err(|e| MigrateError::MissingParameter(e.to_string()))?;
            new_sql.push_str(&replacement);
            last_match = m.end();
        }
        new_sql.push_str(&sql[last_match..]);
        Ok(Migration {
            version: *version,
            description: description.clone(),
            migration_type: *migration_type,
            sql: Cow::Owned(new_sql),
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

#[cfg(test)]
mod test {
    use super::*;

    use std::env;

    #[test]
    fn test_migration_process_parameters_with_envsub() -> Result<(), MigrateError> {
        const CREATE_USER: &str = r#"
            -- +sqlx envsub on
            CREATE USER '${envsub_test_user}';
            -- +sqlx envsub off
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
            -- +sqlx envsub on
            DROP USER '${envsub_test_user}'; 
            -- +sqlx envsub off
        "#;
        const EXPECTED_RESULT: &str = r#"
            -- +sqlx envsub on
            CREATE USER 'my_user';
            -- +sqlx envsub off
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
            -- +sqlx envsub on
            DROP USER 'my_user'; 
            -- +sqlx envsub off
        "#;

        env::set_var("envsub_test_user", "my_user");
        let migration = Migration::new(
            1,
            Cow::Owned("test a simple envsub".to_string()),
            crate::migrate::MigrationType::Simple,
            Cow::Owned(CREATE_USER.to_string()),
            true,
        );
        let result = migration.process_parameters()?;
        assert_eq!(result.sql, EXPECTED_RESULT);
        Ok(())
    }

    #[test]
    fn test_migration_process_parameters_no_envsub() -> Result<(), MigrateError> {
        const CREATE_TABLE: &str = r#"
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT
            );
        "#;
        let migration = Migration::new(
            1,
            std::borrow::Cow::Owned("test a simple envsub".to_string()),
            crate::migrate::MigrationType::Simple,
            Cow::Owned(CREATE_TABLE.to_string()),
            true,
        );
        let result = migration.process_parameters()?;
        assert_eq!(result.sql, CREATE_TABLE);
        Ok(())
    }

    #[test]
    fn test_migration_process_parameters_missing_env_var() -> Result<(), MigrateError> {
        const CREATE_TABLE: &str = r#"
            -- +sqlx envsub on
            CREATE TABLE foo (
                id BIG SERIAL PRIMARY KEY
                foo TEXT,
                field ${TEST_MISSING_ENV_VAR_FIELD}
            );
            -- +sqlx envsub off
        "#;
        env::set_var("envsub_test_user", "my_user");
        let migration = Migration::new(
            1,
            Cow::Owned("test a simple envsub".to_string()),
            crate::migrate::MigrationType::Simple,
            Cow::Owned(CREATE_TABLE.to_string()),
            true,
        );
        let Err(MigrateError::MissingParameter(_)) = migration.process_parameters() else {
            panic!("Missing env var not caught in process parameters missing env var")
        };
        Ok(())
    }
}
