use sqlx_core::{connection::LogSettings, logger};
use std::collections::HashSet;
use std::fmt::Debug;

pub(crate) use sqlx_core::logger::*;

pub struct QueryPlanLogger<'q, T: Debug + 'static, R: Debug + 'static, P: Debug> {
    sql: &'q str,
    unknown_operations: HashSet<usize>,
    table_info: Vec<Option<T>>,
    results: Vec<(Vec<usize>, Option<R>)>,
    program: &'q [P],
    settings: LogSettings,
}

impl<T: Debug, R: Debug, P: Debug> core::fmt::Display for QueryPlanLogger<'_, T, R, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //writes query plan history in dot format
        f.write_str("digraph {")?;
        for (idx, instruction) in self.program.iter().enumerate() {
            let escaped_instruction = format!("{:?}", instruction)
                .replace("\\", "\\\\")
                .replace("\"", "'");
            write!(f, "{} [label=\"{}\"", idx, escaped_instruction)?;

            if self.unknown_operations.contains(&idx) {
                f.write_str(" style=dashed")?;
            }

            f.write_str("];\n")?;
        }

        f.write_str("subgraph table_info {\n")?;
        f.write_str("node [shape=box];\n")?;
        for (idx, table_info_option) in self.table_info.iter().enumerate() {
            if let Some(table_info) = table_info_option {
                let escaped_data = format!("{:?}", table_info)
                    .replace("\\", "\\\\")
                    .replace("\"", "'");
                write!(
                    f,
                    "{} -> table{}; table{} [label=\"{}\"];\n",
                    idx, idx, idx, escaped_data
                )?;
            }
        }
        f.write_str("};\n")?;

        for (idx, (history, result)) in self.results.iter().enumerate() {
            f.write_str("subgraph {")?;

            let color_names = [
                "blue",
                "red",
                "cyan",
                "yellow",
                "green",
                "magenta",
                "orange",
                "purple",
                "orangered",
                "sienna",
                "olivedrab",
                "pink",
            ];
            let color_name_root = color_names[idx % color_names.len()];
            let color_name_suffix = match (idx / color_names.len()) % 4 {
                0 => "1",
                1 => "4",
                2 => "3",
                3 => "2",
                _ => "",
            }; //colors are easily confused after color_names.len() * 2, and outright reused after color_names.len() * 4
            write!(
                f,
                "edge [colorscheme=x11 color={}{} label={}];",
                color_name_root, color_name_suffix, idx
            )?;

            let mut history_iter = history.iter();
            if let Some(item) = history_iter.next() {
                write!(f, "{}", item)?;
                while let Some(item) = history_iter.next() {
                    write!(f, " -> {}", item)?;
                }

                let escaped_result = format!("{:?}", result)
                    .replace("\\", "\\\\")
                    .replace("\"", "'");
                write!(
                    f,
                    " -> \"{}\"; \"{}\" [shape=box];",
                    escaped_result, escaped_result
                )?;
            }
            f.write_str("};\n")?;
        }

        f.write_str("}\n")?;
        Ok(())
    }
}

impl<'q, T: Debug, R: Debug, P: Debug> QueryPlanLogger<'q, T, R, P> {
    pub fn new(sql: &'q str, program: &'q [P], settings: LogSettings) -> Self {
        let mut table_info = Vec::new();
        table_info.resize_with(program.len(), || None);

        Self {
            sql,
            unknown_operations: HashSet::new(),
            table_info,
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

    pub fn add_table_info(&mut self, operation: usize, detail: Option<T>) {
        while self.table_info.len() < operation {
            self.table_info.push(None);
        }
        self.table_info.insert(operation, detail);
    }

    pub fn add_result(&mut self, result: (Vec<usize>, Option<R>)) {
        self.results.push(result);
    }

    pub fn add_unknown_operation(&mut self, operation: usize) {
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

                sqlx_core::private_tracing_dynamic_event!(
                    target: "sqlx::explain",
                    tracing_level,
                    "{}; program:\n{}\n\n{:?}", summary, self, sql
                );
            }
        }
    }
}

impl<'q, T: Debug, R: Debug, P: Debug> Drop for QueryPlanLogger<'q, T, R, P> {
    fn drop(&mut self) {
        self.finish();
    }
}
