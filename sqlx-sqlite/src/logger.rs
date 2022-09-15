use sqlx_core::connection::LogSettings;
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

    pub fn add_result(&mut self, result: R) {
        self.results.push(result);
    }

    pub fn add_unknown_operation(&mut self, operation: O) {
        self.unknown_operations.insert(operation);
    }

    pub fn finish(&self) {
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

impl<'q, O: Debug + Hash + Eq, R: Debug, P: Debug> Drop for QueryPlanLogger<'q, O, R, P> {
    fn drop(&mut self) {
        self.finish();
    }
}
