use std::future::Future;
use std::ops::Deref;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

use crate::error::Error;
use crate::executor::Executor;
use crate::pool::{Pool, PoolOptions};
use crate::query::query;
use crate::{MySql, MySqlConnectOptions, MySqlConnection, MySqlDatabaseError};
use sqlx_core::connection::{ConnectOptions, Connection};
use sqlx_core::query_builder::QueryBuilder;
use sqlx_core::query_scalar::query_scalar;
use sqlx_core::sql_str::AssertSqlSafe;
use sqlx_core::testing::{migrations_hash, template_db_name};

pub(crate) use sqlx_core::testing::*;

// Using a blocking `OnceLock` here because the critical sections are short.
static MASTER_POOL: OnceLock<Pool<MySql>> = OnceLock::new();

/// Environment variable to disable template cloning.
const SQLX_TEST_NO_TEMPLATE: &str = "SQLX_TEST_NO_TEMPLATE";

/// Check if template cloning is enabled.
fn templates_enabled() -> bool {
    std::env::var(SQLX_TEST_NO_TEMPLATE).is_err()
}

/// Get or create a template database with migrations applied.
/// Returns the template database name if successful, or None if templates are disabled.
async fn get_or_create_template(
    conn: &mut MySqlConnection,
    master_opts: &MySqlConnectOptions,
    migrator: &sqlx_core::migrate::Migrator,
) -> Result<Option<String>, Error> {
    if !templates_enabled() {
        return Ok(None);
    }

    let hash = migrations_hash(migrator);
    let tpl_name = template_db_name(&hash);

    // Use MySQL's GET_LOCK for synchronization across processes
    // Timeout of -1 means wait indefinitely
    query("SELECT GET_LOCK(?, -1)")
        .bind("sqlx_template_lock")
        .execute(&mut *conn)
        .await?;

    // Ensure template tracking table exists
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_test_templates (
            template_name VARCHAR(255) PRIMARY KEY,
            migrations_hash VARCHAR(64) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            UNIQUE KEY (migrations_hash)
        )
        "#,
    )
    .await?;

    // Check if template already exists in tracking table
    let existing: Option<String> =
        query_scalar("SELECT template_name FROM _sqlx_test_templates WHERE migrations_hash = ?")
            .bind(&hash)
            .fetch_optional(&mut *conn)
            .await?;

    if let Some(existing_name) = existing {
        // Template exists, update last_used_at and return
        query("UPDATE _sqlx_test_templates SET last_used_at = CURRENT_TIMESTAMP WHERE template_name = ?")
            .bind(&existing_name)
            .execute(&mut *conn)
            .await?;

        // Release lock
        query("SELECT RELEASE_LOCK(?)")
            .bind("sqlx_template_lock")
            .execute(&mut *conn)
            .await?;

        return Ok(Some(existing_name));
    }

    // Create new template database (use IF NOT EXISTS for idempotency)
    // The database might exist from a previous run without being registered
    conn.execute(AssertSqlSafe(format!(
        "CREATE DATABASE IF NOT EXISTS `{tpl_name}`"
    )))
    .await?;

    // Check if this is a fresh database or one left over from a previous run
    // by checking if it already has migrations recorded
    let template_opts = master_opts.clone().database(&tpl_name);
    let mut template_conn: MySqlConnection = template_opts.connect().await?;

    // Try to count migrations - if the table doesn't exist or is empty, we need to run migrations
    let migration_count: Result<i64, _> = query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&mut template_conn)
        .await;

    let needs_migrations = match migration_count {
        Ok(count) => count == 0, // Table exists but is empty
        Err(_) => true,          // Table doesn't exist (error 1146) or other error
    };

    // Only run migrations if the database is fresh (no migrations table)
    if needs_migrations {
        if let Err(e) = migrator.run_direct(None, &mut template_conn).await {
            // Clean up on failure
            template_conn.close().await.ok();
            conn.execute(AssertSqlSafe(format!(
                "DROP DATABASE IF EXISTS `{tpl_name}`"
            )))
            .await
            .ok();
            query("SELECT RELEASE_LOCK(?)")
                .bind("sqlx_template_lock")
                .execute(&mut *conn)
                .await?;
            return Err(Error::Protocol(format!(
                "Failed to apply migrations to template: {}",
                e
            )));
        }
    }

    template_conn.close().await?;

    // Register template (use INSERT IGNORE in case it was already registered by another process)
    query("INSERT IGNORE INTO _sqlx_test_templates (template_name, migrations_hash) VALUES (?, ?)")
        .bind(&tpl_name)
        .bind(&hash)
        .execute(&mut *conn)
        .await?;

    // Release lock
    query("SELECT RELEASE_LOCK(?)")
        .bind("sqlx_template_lock")
        .execute(&mut *conn)
        .await?;

    eprintln!("created template database {tpl_name}");

    Ok(Some(tpl_name))
}

/// Clone a template database to a new test database using mysqldump.
/// Falls back to in-process schema copy if mysqldump is not available.
async fn clone_database(
    conn: &mut MySqlConnection,
    master_opts: &MySqlConnectOptions,
    template_name: &str,
    new_db_name: &str,
) -> Result<(), Error> {
    // First, create the new empty database
    conn.execute(AssertSqlSafe(format!("CREATE DATABASE `{new_db_name}`")))
        .await?;

    // Try mysqldump approach first (faster for large schemas)
    if clone_with_mysqldump(master_opts, template_name, new_db_name).is_ok() {
        return Ok(());
    }

    // Fall back to in-process schema copy
    clone_in_process(conn, master_opts, template_name, new_db_name).await
}

/// Clone database using mysqldump and mysql commands.
/// Uses synchronous process execution for cross-runtime compatibility.
fn clone_with_mysqldump(
    opts: &MySqlConnectOptions,
    template_name: &str,
    new_db_name: &str,
) -> Result<(), Error> {
    use std::io::Write;
    use std::process::Command;

    let host = &opts.host;
    let port = opts.port.to_string();
    let user = &opts.username;

    // Build mysqldump command
    let mut dump_cmd = Command::new("mysqldump");
    dump_cmd
        .arg("--no-data")
        .arg("--routines")
        .arg("--triggers")
        .arg("-h")
        .arg(host)
        .arg("-P")
        .arg(&port)
        .arg("-u")
        .arg(user);

    if let Some(ref password) = opts.password {
        dump_cmd.arg(format!("-p{}", password));
    }

    dump_cmd.arg(template_name);
    dump_cmd.stdout(Stdio::piped());
    dump_cmd.stderr(Stdio::null());

    let dump_output = dump_cmd
        .output()
        .map_err(|e| Error::Protocol(format!("Failed to run mysqldump: {}", e)))?;

    if !dump_output.status.success() {
        return Err(Error::Protocol("mysqldump failed".into()));
    }

    // Build mysql command to import
    let mut import_cmd = Command::new("mysql");
    import_cmd
        .arg("-h")
        .arg(host)
        .arg("-P")
        .arg(&port)
        .arg("-u")
        .arg(user);

    if let Some(ref password) = opts.password {
        import_cmd.arg(format!("-p{}", password));
    }

    import_cmd.arg(new_db_name);
    import_cmd.stdin(Stdio::piped());
    import_cmd.stdout(Stdio::null());
    import_cmd.stderr(Stdio::null());

    let mut import_child = import_cmd
        .spawn()
        .map_err(|e| Error::Protocol(format!("Failed to spawn mysql: {}", e)))?;

    // Write dump output to mysql stdin
    if let Some(ref mut stdin) = import_child.stdin {
        stdin
            .write_all(&dump_output.stdout)
            .map_err(|e| Error::Protocol(format!("Failed to write to mysql stdin: {}", e)))?;
    }

    let import_status = import_child
        .wait()
        .map_err(|e| Error::Protocol(format!("Failed to wait for mysql: {}", e)))?;

    if !import_status.success() {
        return Err(Error::Protocol("mysql import failed".into()));
    }

    Ok(())
}

/// Clone database using in-process SQL commands (fallback).
async fn clone_in_process(
    conn: &mut MySqlConnection,
    master_opts: &MySqlConnectOptions,
    template_name: &str,
    new_db_name: &str,
) -> Result<(), Error> {
    // Get all tables from template
    let tables: Vec<String> = query_scalar(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = ? AND table_type = 'BASE TABLE'",
    )
    .bind(template_name)
    .fetch_all(&mut *conn)
    .await?;

    // Connect to the new database for copying
    let new_db_opts = master_opts.clone().database(new_db_name);
    let mut new_conn: MySqlConnection = new_db_opts.connect().await?;

    for table in &tables {
        // Copy table structure
        new_conn
            .execute(AssertSqlSafe(format!(
                "CREATE TABLE `{new_db_name}`.`{table}` LIKE `{template_name}`.`{table}`"
            )))
            .await?;

        // Copy table data (for migrations table, etc.)
        new_conn
            .execute(AssertSqlSafe(format!(
                "INSERT INTO `{new_db_name}`.`{table}` SELECT * FROM `{template_name}`.`{table}`"
            )))
            .await?;
    }

    new_conn.close().await?;

    Ok(())
}

impl TestSupport for MySql {
    fn test_context(
        args: &TestArgs,
    ) -> impl Future<Output = Result<TestContext<Self>, Error>> + Send + '_ {
        test_context(args)
    }

    async fn cleanup_test(db_name: &str) -> Result<(), Error> {
        let mut conn = MASTER_POOL
            .get()
            .expect("cleanup_test() invoked outside `#[sqlx::test]`")
            .acquire()
            .await?;

        do_cleanup(&mut conn, db_name).await
    }

    async fn cleanup_test_dbs() -> Result<Option<usize>, Error> {
        let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let mut conn = MySqlConnection::connect(&url).await?;

        let delete_db_names: Vec<String> = query_scalar("select db_name from _sqlx_test_databases")
            .fetch_all(&mut conn)
            .await?;

        if delete_db_names.is_empty() {
            return Ok(None);
        }

        let mut deleted_db_names = Vec::with_capacity(delete_db_names.len());

        let mut builder = QueryBuilder::new("drop database if exists ");

        for db_name in &delete_db_names {
            builder.push(db_name);

            match builder.build().execute(&mut conn).await {
                Ok(_deleted) => {
                    deleted_db_names.push(db_name);
                }
                // Assume a database error just means the DB is still in use.
                Err(Error::Database(dbe)) => {
                    eprintln!("could not clean test database {db_name:?}: {dbe}")
                }
                // Bubble up other errors
                Err(e) => return Err(e),
            }

            builder.reset();
        }

        if deleted_db_names.is_empty() {
            return Ok(None);
        }

        let mut query = QueryBuilder::new("delete from _sqlx_test_databases where db_name in (");

        let mut separated = query.separated(",");

        for db_name in &deleted_db_names {
            separated.push_bind(db_name);
        }

        query.push(")").build().execute(&mut conn).await?;

        let _ = conn.close().await;
        Ok(Some(delete_db_names.len()))
    }

    async fn snapshot(_conn: &mut Self::Connection) -> Result<FixtureSnapshot<Self>, Error> {
        // TODO: I want to get the testing feature out the door so this will have to wait,
        // but I'm keeping the code around for now because I plan to come back to it.
        todo!()
    }
}

async fn test_context(args: &TestArgs) -> Result<TestContext<MySql>, Error> {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let master_opts = MySqlConnectOptions::from_str(&url).expect("failed to parse DATABASE_URL");

    let pool = PoolOptions::new()
        // MySql's normal connection limit is 150 plus 1 superuser connection
        // We don't want to use the whole cap and there may be fuzziness here due to
        // concurrently running tests anyway.
        .max_connections(20)
        // Immediately close master connections. Tokio's I/O streams don't like hopping runtimes.
        .after_release(|_conn, _| Box::pin(async move { Ok(false) }))
        .connect_lazy_with(master_opts.clone());

    let master_pool = match once_lock_try_insert_polyfill(&MASTER_POOL, pool) {
        Ok(inserted) => inserted,
        Err((existing, pool)) => {
            // Sanity checks.
            assert_eq!(
                existing.connect_options().host,
                pool.connect_options().host,
                "DATABASE_URL changed at runtime, host differs"
            );

            assert_eq!(
                existing.connect_options().database,
                pool.connect_options().database,
                "DATABASE_URL changed at runtime, database differs"
            );

            existing
        }
    };

    let mut conn = master_pool.acquire().await?;

    cleanup_old_dbs(&mut conn).await?;

    // language=MySQL
    conn.execute(
        r#"
        create table if not exists _sqlx_test_databases (
            db_name text not null,
            test_path text not null,
            created_at timestamp not null default current_timestamp,
            -- BLOB/TEXT columns can only be used as index keys with a prefix length:
            -- https://dev.mysql.com/doc/refman/8.4/en/column-indexes.html#column-indexes-prefix
            primary key(db_name(63))
        );
    "#,
    )
    .await?;

    let db_name = MySql::db_name(args);
    do_cleanup(&mut conn, &db_name).await?;

    query("insert into _sqlx_test_databases(db_name, test_path) values (?, ?)")
        .bind(&db_name)
        .bind(args.test_path)
        .execute(&mut *conn)
        .await?;

    // Try to use template cloning if migrations are provided
    let from_template = if let Some(migrator) = args.migrator {
        match get_or_create_template(&mut conn, &master_opts, migrator).await {
            Ok(Some(template_name)) => {
                // Clone from template (fast path)
                match clone_database(&mut conn, &master_opts, &template_name, &db_name).await {
                    Ok(()) => {
                        eprintln!("cloned database {db_name} from template {template_name}");
                        true
                    }
                    Err(e) => {
                        // Clean up partial database and fall back to empty database
                        eprintln!(
                            "failed to clone template, falling back to empty database: {}",
                            e
                        );
                        conn.execute(AssertSqlSafe(format!(
                            "drop database if exists `{db_name}`"
                        )))
                        .await
                        .ok();
                        conn.execute(AssertSqlSafe(format!("create database `{db_name}`")))
                            .await?;
                        eprintln!("created database {db_name}");
                        false
                    }
                }
            }
            Ok(None) => {
                // Templates disabled or not available
                conn.execute(AssertSqlSafe(format!("create database `{db_name}`")))
                    .await?;
                eprintln!("created database {db_name}");
                false
            }
            Err(e) => {
                // Template creation failed, fall back to empty database
                eprintln!(
                    "failed to create template, falling back to empty database: {}",
                    e
                );
                conn.execute(AssertSqlSafe(format!("create database `{db_name}`")))
                    .await?;
                eprintln!("created database {db_name}");
                false
            }
        }
    } else {
        // No migrations, create empty database
        conn.execute(AssertSqlSafe(format!("create database `{db_name}`")))
            .await?;
        eprintln!("created database {db_name}");
        false
    };

    Ok(TestContext {
        pool_opts: PoolOptions::new()
            // Don't allow a single test to take all the connections.
            // Most tests shouldn't require more than 5 connections concurrently,
            // or else they're likely doing too much in one test.
            .max_connections(5)
            // Close connections ASAP if left in the idle queue.
            .idle_timeout(Some(Duration::from_secs(1)))
            .parent(master_pool.clone()),
        connect_opts: master_pool
            .connect_options()
            .deref()
            .clone()
            .database(&db_name),
        db_name,
        from_template,
    })
}

async fn do_cleanup(conn: &mut MySqlConnection, db_name: &str) -> Result<(), Error> {
    let delete_db_command = format!("drop database if exists {db_name};");
    conn.execute(AssertSqlSafe(delete_db_command)).await?;
    query("delete from _sqlx_test_databases where db_name = ?")
        .bind(db_name)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

async fn cleanup_old_dbs(conn: &mut MySqlConnection) -> Result<(), Error> {
    let res: Result<Vec<u64>, Error> = query_scalar("select db_id from _sqlx_test_databases")
        .fetch_all(&mut *conn)
        .await;

    let db_ids = match res {
        Ok(db_ids) => db_ids,
        Err(e) => {
            if let Some(dbe) = e.as_database_error() {
                match dbe.downcast_ref::<MySqlDatabaseError>().number() {
                    // Column `db_id` does not exist:
                    // https://dev.mysql.com/doc/mysql-errors/8.0/en/server-error-reference.html#error_er_bad_field_error
                    //
                    // The table has already been migrated.
                    1054 => return Ok(()),
                    // Table `_sqlx_test_databases` does not exist.
                    // No cleanup needed.
                    // https://dev.mysql.com/doc/mysql-errors/8.0/en/server-error-reference.html#error_er_no_such_table
                    1146 => return Ok(()),
                    _ => (),
                }
            }

            return Err(e);
        }
    };

    // Drop old-style test databases.
    for id in db_ids {
        match conn
            .execute(AssertSqlSafe(format!(
                "drop database if exists _sqlx_test_database_{id}"
            )))
            .await
        {
            Ok(_deleted) => (),
            // Assume a database error just means the DB is still in use.
            Err(Error::Database(dbe)) => {
                eprintln!("could not clean old test database _sqlx_test_database_{id}: {dbe}");
            }
            // Bubble up other errors
            Err(e) => return Err(e),
        }
    }

    conn.execute("drop table if exists _sqlx_test_databases")
        .await?;

    Ok(())
}

fn once_lock_try_insert_polyfill<T>(this: &OnceLock<T>, value: T) -> Result<&T, (&T, T)> {
    let mut value = Some(value);
    let res = this.get_or_init(|| value.take().unwrap());
    match value {
        None => Ok(res),
        Some(value) => Err((res, value)),
    }
}
