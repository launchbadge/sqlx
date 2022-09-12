use crate::connection::LogSettings;
#[cfg(feature = "sqlite")]
use std::collections::HashSet;
#[cfg(feature = "sqlite")]
use std::fmt::Debug;
#[cfg(feature = "sqlite")]
use std::hash::Hash;
use std::time::Instant;

pub(crate) struct QueryLogger<'q> {
    sql: &'q str,
    rows_returned: u64,
    rows_affected: u64,
    start: Instant,
    settings: LogSettings,
}

impl<'q> QueryLogger<'q> {
    pub(crate) fn new(sql: &'q str, settings: LogSettings) -> Self {
        Self {
            sql,
            rows_returned: 0,
            rows_affected: 0,
            start: Instant::now(),
            settings,
        }
    }

    pub(crate) fn increment_rows_returned(&mut self) {
        self.rows_returned += 1;
    }

    pub(crate) fn increase_rows_affected(&mut self, n: u64) {
        self.rows_affected += n;
    }

    pub(crate) fn finish(&self) {
        let elapsed = self.start.elapsed();

        let lvl = if elapsed >= self.settings.slow_statements_duration {
            self.settings.slow_statements_level
        } else {
            self.settings.statements_level
        };

        if let Some(lvl) = lvl
            .to_level()
            .filter(|lvl| log::log_enabled!(target: "sqlx::query", *lvl))
        {
            let mut summary = parse_query_summary(&self.sql);

            let sql = if summary != self.sql {
                summary.push_str(" …");
                format!(
                    "\n\n{}\n",
                    sqlformat::format(
                        &self.sql,
                        &sqlformat::QueryParams::None,
                        sqlformat::FormatOptions::default()
                    )
                )
            } else {
                String::new()
            };

            log::logger().log(
                &log::Record::builder()
                    .args(format_args!(
                        "{}; rows affected: {}, rows returned: {}, elapsed: {:.3?}{}",
                        summary, self.rows_affected, self.rows_returned, elapsed, sql
                    ))
                    .level(lvl)
                    .module_path_static(Some("sqlx::query"))
                    .target("sqlx::query")
                    .build(),
            );
        }
    }
}

impl<'q> Drop for QueryLogger<'q> {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(feature = "sqlite")]
pub(crate) struct QueryPlanLogger<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> {
    sql: &'q str,
    unknown_operations: HashSet<O>,
    results: Vec<R>,
    program: &'q [P],
    settings: LogSettings,
}

#[cfg(feature = "sqlite")]
impl<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> QueryPlanLogger<'q, O, R, P> {
    pub(crate) fn new(sql: &'q str, program: &'q [P], settings: LogSettings) -> Self {
        Self {
            sql,
            unknown_operations: HashSet::new(),
            results: Vec::new(),
            program,
            settings,
        }
    }

    pub(crate) fn log_enabled(&self) -> bool {
        if let Some(_lvl) = self
            .settings
            .statements_level
            .to_level()
            .filter(|lvl| log::log_enabled!(target: "sqlx::explain", *lvl))
        {
            return true;
        } else {
            return false;
        }
    }

    pub(crate) fn add_result(&mut self, result: R) {
        self.results.push(result);
    }

    pub(crate) fn add_unknown_operation(&mut self, operation: O) {
        self.unknown_operations.insert(operation);
    }

    pub(crate) fn finish(&self) {
        let lvl = self.settings.statements_level;

        if let Some(lvl) = lvl
            .to_level()
            .filter(|lvl| log::log_enabled!(target: "sqlx::explain", *lvl))
        {
            let mut summary = parse_query_summary(&self.sql);

            let sql = if summary != self.sql {
                summary.push_str(" …");
                format!(
                    "\n\n{}\n",
                    sqlformat::format(
                        &self.sql,
                        &sqlformat::QueryParams::None,
                        sqlformat::FormatOptions::default()
                    )
                )
            } else {
                String::new()
            };

            log::logger().log(
                &log::Record::builder()
                    .args(format_args!(
                        "{}; program:{:?}, unknown_operations:{:?}, results: {:?}{}",
                        summary, self.program, self.unknown_operations, self.results, sql
                    ))
                    .level(lvl)
                    .module_path_static(Some("sqlx::explain"))
                    .target("sqlx::explain")
                    .build(),
            );
        }
    }
}

#[cfg(feature = "sqlite")]
impl<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> Drop for QueryPlanLogger<'q, O, R, P> {
    fn drop(&mut self) {
        self.finish();
    }
}

fn parse_query_summary(sql: &str) -> String {
    // For now, just take the first 4 words
    sql.split_whitespace()
        .take(4)
        .collect::<Vec<&str>>()
        .join(" ")
}
