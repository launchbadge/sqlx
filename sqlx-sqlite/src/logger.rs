use sqlx_core::{connection::LogSettings, logger};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

pub(crate) use sqlx_core::logger::*;

pub struct QueryPlanLogger<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> {
    sql: &'q str,
    unknown_operations: HashSet<O>,
    results: Vec<R>,
    program: &'q [P],
    settings: LogSettings,
}

impl<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> QueryPlanLogger<'q, O, R, P> {
    pub fn new(sql: &'q str, program: &'q [P], settings: LogSettings) -> Self {
        Self {
            sql,
            unknown_operations: HashSet::new(),
            results: Vec::new(),
            program,
            settings,
        }
    }

    pub fn log_enabled(&self) -> bool {
        if let Some((tracing_level, log_level)) =
            logger::private_level_filter_to_levels(self.settings.statements_level)
        {
            log::log_enabled!(log_level)
                || sqlx_core::private_tracing_dynamic_enabled!(tracing_level)
        } else {
            false
        }
    }

    pub fn add_result(&mut self, result: R) {
        self.results.push(result);
    }

    pub fn add_unknown_operation(&mut self, operation: O) {
        self.unknown_operations.insert(operation);
    }

    pub fn finish(&self) {
        let lvl = self.settings.statements_level;

        if let Some((tracing_level, log_level)) = logger::private_level_filter_to_levels(lvl) {
            let log_is_enabled = log::log_enabled!(target: "sqlx::explain", log_level)
                || private_tracing_dynamic_enabled!(target: "sqlx::explain", tracing_level);
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

                sqlx_core::private_tracing_dynamic_event!(
                    target: "sqlx::explain",
                    tracing_level,
                    message,
                );
            }
        }
    }
}

impl<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> Drop for QueryPlanLogger<'q, O, R, P> {
    fn drop(&mut self) {
        self.finish();
    }
}
