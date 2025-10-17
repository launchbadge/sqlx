use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::executor::Executor;
use crate::{MySqlConnectOptions, MySqlConnection};
use log::LevelFilter;
use sqlx_core::sql_str::AssertSqlSafe;
use sqlx_core::Url;
use std::time::Duration;
// wasm-specific runtime helpers are available via `sqlx_core::rt::wasm_worker`.

impl ConnectOptions for MySqlConnectOptions {
    type Connection = MySqlConnection;

    fn from_url(url: &Url) -> Result<Self, Error> {
        Self::parse_from_url(url)
    }

    fn to_url_lossy(&self) -> Url {
        self.build_url()
    }

    async fn connect(&self) -> Result<Self::Connection, Error>
    where
        Self::Connection: Sized + Send + 'static,
    {
        // On wasm, the MySQL connection future may contain non-Send internals from
        // wasip3/wit-bindgen. Run the connection/initialization on the wasip3 async
        // runtime using `async_support::spawn` and communicate the result back over
        // a tokio oneshot channel. The returned future (awaiting the oneshot) is
        // Send, so callers that require Send are satisfied.
        let options = self.clone();

        // On wasm we must dispatch to the single-threaded wasip3 runtime so
        // that any `!Send` futures from wit-bindgen do not escape the local
        // runtime. On non-wasm targets we can just run the logic directly.
        #[cfg(target_arch = "wasm32")]
        {
            let conn_res: Result<MySqlConnection, Error> =
                sqlx_core::rt::wasm_worker::dispatch(move || async move {
                    let mut conn = MySqlConnection::establish(&options).await?;

                    let mut sql_mode = Vec::new();
                    if options.pipes_as_concat {
                        sql_mode.push(r#"PIPES_AS_CONCAT"#);
                    }
                    if options.no_engine_substitution {
                        sql_mode.push(r#"NO_ENGINE_SUBSTITUTION"#);
                    }

                    let mut opts = Vec::new();
                    if !sql_mode.is_empty() {
                        opts.push(format!(
                            r#"sql_mode=(SELECT CONCAT(@@sql_mode, ',{}'))"#,
                            sql_mode.join(",")
                        ));
                    }

                    if let Some(timezone) = &options.timezone {
                        opts.push(format!(r#"time_zone='{}'"#, timezone));
                    }

                    if options.set_names {
                        let set_names = if let Some(collation) = &options.collation {
                            format!(r#"NAMES {} COLLATE {collation}"#, options.charset,)
                        } else {
                            format!("NAMES {}", options.charset)
                        };
                        opts.push(set_names);
                    }

                    if !opts.is_empty() {
                        conn.execute(AssertSqlSafe(format!(r#"SET {};"#, opts.join(","))))
                            .await?;
                    }

                    Ok(conn)
                })
                .await;

            conn_res
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut conn = MySqlConnection::establish(&options).await?;

            let mut sql_mode = Vec::new();
            if options.pipes_as_concat {
                sql_mode.push(r#"PIPES_AS_CONCAT"#);
            }
            if options.no_engine_substitution {
                sql_mode.push(r#"NO_ENGINE_SUBSTITUTION"#);
            }

            let mut opts = Vec::new();
            if !sql_mode.is_empty() {
                opts.push(format!(
                    r#"sql_mode=(SELECT CONCAT(@@sql_mode, ',{}'))"#,
                    sql_mode.join(",")
                ));
            }

            if let Some(timezone) = &options.timezone {
                opts.push(format!(r#"time_zone='{}'"#, timezone));
            }

            if options.set_names {
                let set_names = if let Some(collation) = &options.collation {
                    format!(r#"NAMES {} COLLATE {collation}"#, options.charset,)
                } else {
                    format!("NAMES {}", options.charset)
                };
                opts.push(set_names);
            }

            if !opts.is_empty() {
                conn.execute(AssertSqlSafe(format!(r#"SET {};"#, opts.join(","))))
                    .await?;
            }

            Ok(conn)
        }
    }

    fn log_statements(mut self, level: LevelFilter) -> Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(mut self, level: LevelFilter, duration: Duration) -> Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
