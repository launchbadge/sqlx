use sqlx_core::{connection::LogSettings, logger};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

pub(crate) use sqlx_core::logger::*;

#[derive(Debug)]
pub(crate) enum BranchResult<R: Debug + 'static> {
    Result(R),
    Dedup(BranchParent),
    Halt,
    Error,
    GasLimit,
    LoopLimit,
    Branched,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BranchParent {
    pub id: usize,
    pub program_i: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BranchHistory {
    pub id: usize,
    pub parent: Option<BranchParent>,
    pub program_i: Vec<usize>,
}

pub struct QueryPlanLogger<'q, T: Debug + 'static, R: Debug + 'static, P: Debug> {
    sql: &'q str,
    unknown_operations: HashSet<usize>,
    table_info: Vec<(BranchParent, T)>,
    results: Vec<(BranchHistory, BranchResult<R>)>,
    program: &'q [P],
    settings: LogSettings,
}

impl<T: Debug, R: Debug, P: Debug> core::fmt::Display for QueryPlanLogger<'_, T, R, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //writes query plan history in dot format
        f.write_str("digraph {\n")?;

        f.write_str("subgraph operations {\n")?;
        f.write_str("style=\"rounded\";\nnode [shape=\"point\"];\n")?;

        //using BTreeMap for predictable ordering
        let mut instruction_uses: std::collections::BTreeMap<usize, BTreeSet<usize>> =
            Default::default();

        for (program_i, id) in self.results.iter().flat_map(|(history, _)| {
            history
                .program_i
                .iter()
                .map(|program_i| (*program_i, history.id))
        }) {
            instruction_uses.entry(program_i).or_default().insert(id);
        }

        for (idx, instruction) in self.program.iter().enumerate() {
            let escaped_instruction = format!("{:?}", instruction)
                .replace("\\", "\\\\")
                .replace("\"", "'");
            write!(
                f,
                "subgraph cluster_{} {{ label=\"{}\"",
                idx, escaped_instruction
            )?;

            if self.unknown_operations.contains(&idx) {
                f.write_str(" style=dashed")?;
            }

            f.write_str(";\n")?;

            for id in instruction_uses.entry(idx).or_default().iter() {
                write!(f, "\"b{}p{}\";", id, idx)?;
            }

            f.write_str("}\n")?;
        }
        f.write_str("};\n")?; //subgraph operations

        f.write_str("subgraph table_info {\n")?;
        f.write_str("node [shape=box];\n")?;
        for (idx, (parent, table_info)) in self.table_info.iter().enumerate() {
            let escaped_data = format!("{:?}", table_info)
                .replace("\\", "\\\\")
                .replace("\"", "'");
            write!(
                f,
                "\"b{}p{}\" -> table{}; table{} [label=\"{}\"];\n",
                parent.id, parent.program_i, idx, idx, escaped_data
            )?;
        }
        f.write_str("};\n")?; //subgraph table_info

        f.write_str("subgraph branches {\n")?;

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
                color_name_root, color_name_suffix, history.id
            )?;

            if history.program_i.len() > 0 {
                let mut program_iter = history.program_i.iter();
                if let Some(program_i) = program_iter.next() {
                    if let Some(BranchParent { program_i, id }) = history.parent {
                        write!(f, "\"b{}p{}\"->", id, program_i)?;
                    }
                    write!(f, "\"b{}p{}\"", history.id, program_i)?;
                    while let Some(program_i) = program_iter.next() {
                        write!(f, "->\"b{}p{}\"", history.id, program_i)?;
                    }
                }

                if let Some(id) = history.program_i.last() {
                    if let BranchResult::Dedup(BranchParent {
                        program_i: dedup_program_i,
                        id: dedup_id,
                    }) = result
                    {
                        write!(
                            f,
                            "\"b{}p{}\"->\"b{}p{}\" [style=dotted]",
                            history.id, id, dedup_id, dedup_program_i
                        )?;
                    } else {
                        let escaped_result = format!("{:?}", result)
                            .replace("\\", "\\\\")
                            .replace("\"", "'");
                        write!(
                            f,
                            " -> \"{}\"; \"{}\" [shape=box];",
                            escaped_result, escaped_result
                        )?;
                    }
                }
            }
            f.write_str("};\n")?;
        }
        f.write_str("};\n")?; //branches

        f.write_str("}\n")?;
        Ok(())
    }
}

impl<'q, T: Debug, R: Debug, P: Debug> QueryPlanLogger<'q, T, R, P> {
    pub fn new(sql: &'q str, program: &'q [P], settings: LogSettings) -> Self {
        Self {
            sql,
            unknown_operations: HashSet::new(),
            table_info: Vec::new(),
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

    pub fn add_table_info(&mut self, parent: BranchParent, detail: T) {
        self.table_info.push((parent, detail));
    }

    pub fn add_result(&mut self, history: BranchHistory, result: BranchResult<R>) {
        //don't record any deduplicated branches that didn't execute any instructions
        self.results.push((history, result));
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
