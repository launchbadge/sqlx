use crate::connection::LogSettings;
use std::time::Instant;

// Yes these look silly. `tracing` doesn't currently support dynamic levels
// https://github.com/tokio-rs/tracing/issues/372
#[doc(hidden)]
#[macro_export]
macro_rules! private_tracing_dynamic_enabled {
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
    ($level:expr) => {{
        $crate::private_tracing_dynamic_enabled!(target: module_path!(), $level)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! private_tracing_dynamic_event {
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

#[doc(hidden)]
#[macro_export]
macro_rules! private_tracing_dynamic_span {
    (target: $target:expr, $level:expr, $($args:tt)*) => {{
        use ::tracing::Level;

        match $level {
            Level::ERROR => ::tracing::span!(target: $target, Level::ERROR, $($args)*),
            Level::WARN => ::tracing::span!(target: $target, Level::WARN, $($args)*),
            Level::INFO => ::tracing::span!(target: $target, Level::INFO, $($args)*),
            Level::DEBUG => ::tracing::span!(target: $target, Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::span!(target: $target, Level::TRACE, $($args)*),
        }
    }};
}

#[doc(hidden)]
pub fn private_level_filter_to_levels(
    filter: log::LevelFilter,
) -> Option<(tracing::Level, log::Level)> {
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

pub(crate) fn private_level_filter_to_trace_level(
    filter: log::LevelFilter,
) -> Option<tracing::Level> {
    private_level_filter_to_levels(filter).map(|(level, _)| level)
}

pub use sqlformat;

pub struct QueryLogger<'q> {
    sql: &'q str,
    rows_returned: u64,
    rows_affected: u64,
    start: Instant,
    settings: LogSettings,
    pub span: tracing::Span,
}

impl<'q> QueryLogger<'q> {
    pub fn new(sql: &'q str, db_system: &'q str, settings: LogSettings) -> Self {
        let span = Self::new_tracing_span(db_system, &settings);

        Self {
            sql,
            rows_returned: 0,
            rows_affected: 0,
            start: Instant::now(),
            settings,
            span,
        }
    }

    fn new_tracing_span(db_system: &str, settings: &LogSettings) -> tracing::Span {
        // only create a usable span if the span level is set (i.e. not 'OFF') and
        // the filter would output it
        if let Some(level) = private_level_filter_to_trace_level(settings.span_level) {
            if private_tracing_dynamic_enabled!(target: "sqlx::query", level) {
                use tracing::field::Empty;

                return private_tracing_dynamic_span!(
                    target: "sqlx::query",
                    level,
                    "sqlx::query",
                    summary = Empty,
                    db.statement = Empty,
                    db.system = db_system,
                    rows_affected = Empty,
                    rows_returned = Empty,
                );
            }
        }

        // No-op span preferred over Option<Span>
        tracing::Span::none()
    }

    pub fn increment_rows_returned(&mut self) {
        self.rows_returned += 1;
    }

    pub fn increase_rows_affected(&mut self, n: u64) {
        self.rows_affected += n;
    }

    pub fn finish(&self) {
        let elapsed = self.start.elapsed();

        let was_slow = elapsed >= self.settings.slow_statements_duration;

        let lvl = if was_slow {
            self.settings.slow_statements_level
        } else {
            self.settings.statements_level
        };

        let span_level = private_level_filter_to_trace_level(self.settings.span_level);
        let log_levels = private_level_filter_to_levels(lvl);

        let span_is_enabled = span_level
            .map(|level| private_tracing_dynamic_enabled!(target: "sqlx::query", level))
            .unwrap_or(false);

        let log_is_enabled = log_levels
            .map(|(tracing_level, log_level)| {
                // The enabled level could be set from either tracing world or log world, so check both
                // to see if logging should be enabled for our level
                log::log_enabled!(target: "sqlx::query", log_level)
                    || private_tracing_dynamic_enabled!(target: "sqlx::query", tracing_level)
            })
            .unwrap_or(false);

        // only do these potentially expensive operations if the span or log will record them
        if span_is_enabled || log_is_enabled {
            let mut summary = parse_query_summary(self.sql);

            let sql = if summary != self.sql {
                summary.push_str(" …");
                format!(
                    "\n\n{}\n",
                    sqlformat::format(
                        self.sql,
                        &sqlformat::QueryParams::None,
                        sqlformat::FormatOptions::default()
                    )
                )
            } else {
                String::new()
            };

            // in the case where span_is_enabled is false, these will no-op
            self.span.record("summary", &summary);
            self.span.record("db.statement", &sql);
            self.span.record("rows_affected", self.rows_affected);
            self.span.record("rows_returned", self.rows_returned);

            let _e = self.span.enter();

            if log_is_enabled {
                let (tracing_level, _) = log_levels.unwrap(); // previously checked to be some

                if was_slow {
                    private_tracing_dynamic_event!(
                        target: "sqlx::query",
                        tracing_level,
                        summary,
                        db.statement = sql,
                        rows_affected = self.rows_affected,
                        rows_returned = self.rows_returned,
                        // Human-friendly - includes units (usually ms). Also kept for backward compatibility
                        ?elapsed,
                        // Search friendly - numeric
                        elapsed_secs = elapsed.as_secs_f64(),
                        // When logging to JSON, one can trigger alerts from the presence of this field.
                        slow_threshold=?self.settings.slow_statements_duration,
                        // Make sure to use "slow" in the message as that's likely
                        // what people will grep for.
                        "slow statement: execution time exceeded alert threshold"
                    );
                } else {
                    private_tracing_dynamic_event!(
                        target: "sqlx::query",
                        tracing_level,
                        summary,
                        db.statement = sql,
                        rows_affected = self.rows_affected,
                        rows_returned = self.rows_returned,
                        // Human-friendly - includes units (usually ms). Also kept for backward compatibility
                        ?elapsed,
                        // Search friendly - numeric
                        elapsed_secs = elapsed.as_secs_f64(),
                    );
                }
            }
        }
    }
}

impl<'q> Drop for QueryLogger<'q> {
    fn drop(&mut self) {
        self.finish();
    }
}

pub fn parse_query_summary(sql: &str) -> String {
    // For now, just take the first 4 words
    sql.split_whitespace()
        .take(4)
        .collect::<Vec<&str>>()
        .join(" ")
}
