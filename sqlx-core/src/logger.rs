use crate::connection::LogSettings;
#[cfg(feature = "sqlite")]
use std::collections::HashSet;
#[cfg(feature = "sqlite")]
use std::fmt::Debug;
#[cfg(feature = "sqlite")]
use std::hash::Hash;
use std::time::Instant;

// Yes these look silly. `tracing` doesn't currently support dynamic levels
// https://github.com/tokio-rs/tracing/issues/372
macro_rules! tracing_dynamic_enabled {
    (target: $target:expr, $level:expr) => {{
        use ::tracing::Level;

        match $level {
            Level::ERROR => ::tracing::enabled!(target: $target, Level::ERROR),
            Level::WARN => ::tracing::enabled!(target: $target, Level::WARN),
            Level::INFO => ::tracing::enabled!(target: $target, Level::INFO),
            Level::DEBUG => ::tracing::enabled!(target: $target, Level::DEBUG),
            Level::TRACE => ::tracing::enabled!(target: $target, Level::TRACE),
        }
    }};
}

macro_rules! tracing_dynamic_event {
    (target: $target:expr, $level:expr, $($args:tt)*) => {{
        use ::tracing::Level;

        match $level {
            Level::ERROR => ::tracing::event!(target: $target, Level::ERROR, $($args)*),
            Level::WARN => ::tracing::event!(target: $target, Level::WARN, $($args)*),
            Level::INFO => ::tracing::event!(target: $target, Level::INFO, $($args)*),
            Level::DEBUG => ::tracing::event!(target: $target, Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::event!(target: $target, Level::TRACE, $($args)*),
        }
    }};
}

fn level_filter_to_levels(filter: log::LevelFilter) -> Option<(tracing::Level, log::Level)> {
    let tracing_level = match filter {
        log::LevelFilter::Error => Some(tracing::Level::ERROR),
        log::LevelFilter::Warn => Some(tracing::Level::WARN),
        log::LevelFilter::Info => Some(tracing::Level::INFO),
        log::LevelFilter::Debug => Some(tracing::Level::DEBUG),
        log::LevelFilter::Trace => Some(tracing::Level::TRACE),
        log::LevelFilter::Off => None,
    };

    tracing_level.zip(filter.to_level())
}

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

        if let Some((tracing_level, log_level)) = level_filter_to_levels(lvl) {
            // The enabled level could be set from either tracing world or log world, so check both
            // to see if logging should be enabled for our level
            let log_is_enabled = tracing_dynamic_enabled!(target: "sqlx::query", tracing_level)
                || log::log_enabled!(target: "sqlx::query", log_level);
            if log_is_enabled {
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

                let message = format!(
                    "{}; rows affected: {}, rows returned: {}, elapsed: {:.3?}{}",
                    summary, self.rows_affected, self.rows_returned, elapsed, sql
                );

                tracing_dynamic_event!(target: "sqlx::query", tracing_level, message);
            }
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
        level_filter_to_levels(self.settings.statements_level)
            .map(|(tracing_level, log_level)| {
                tracing_dynamic_enabled!(target: "sqlx::query", tracing_level)
                    || log::log_enabled!(target: "sqlx::query", log_level)
            })
            .unwrap_or_default()
    }

    pub(crate) fn add_result(&mut self, result: R) {
        self.results.push(result);
    }

    pub(crate) fn add_unknown_operation(&mut self, operation: O) {
        self.unknown_operations.insert(operation);
    }

    pub(crate) fn finish(&self) {
        let lvl = self.settings.statements_level;

        if let Some((tracing_level, log_level)) = level_filter_to_levels(lvl) {
            let log_is_enabled = tracing_dynamic_enabled!(target: "sqlx::explain", tracing_level)
                || log::log_enabled!(target: "sqlx::explain", log_level);
            if log_is_enabled {
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

                let message = format!(
                    "{}; program:{:?}, unknown_operations:{:?}, results: {:?}{}",
                    summary, self.program, self.unknown_operations, self.results, sql
                );

                tracing_dynamic_event!(target: "sqlx::explain", tracing_level, message);
            }
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
