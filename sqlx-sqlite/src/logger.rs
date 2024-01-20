use sqlx_core::{connection::LogSettings, logger};
use std::collections::{BTreeMap, HashSet};
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub(crate) struct BranchParent {
    pub id: usize,
    pub idx: usize,
}

#[derive(Debug)]
pub(crate) struct BranchHistory<S: Debug + DebugDiff> {
    pub id: usize,
    pub parent: Option<BranchParent>,
    pub program_i: Vec<InstructionHistory<S>>,
}

#[derive(Debug)]
pub(crate) struct InstructionHistory<S: Debug + DebugDiff> {
    pub program_i: usize,
    pub state: S,
}

pub(crate) trait DebugDiff {
    fn diff(&self, prev: &Self) -> String;
}

pub struct QueryPlanLogger<'q, R: Debug + 'static, S: Debug + DebugDiff + 'static, P: Debug> {
    sql: &'q str,
    unknown_operations: HashSet<usize>,
    results: Vec<(BranchHistory<S>, BranchResult<R>)>,
    program: &'q [P],
    settings: LogSettings,
}

impl<R: Debug, S: Debug + DebugDiff, P: Debug> core::fmt::Display for QueryPlanLogger<'_, R, S, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //writes query plan history in dot format
        f.write_str("digraph {\n")?;

        f.write_str("subgraph operations {\n")?;
        f.write_str("style=\"rounded\";\nnode [shape=\"point\"];\n")?;

        //using BTreeMap for predictable ordering
        let mut instruction_uses: BTreeMap<usize, Vec<BranchParent>> = Default::default();

        for (history, _) in self.results.iter() {
            for (idx, program_i) in history.program_i.iter().enumerate() {
                let references = instruction_uses.entry(program_i.program_i).or_default();
                references.push(BranchParent {
                    id: history.id,
                    idx,
                });
            }
        }

        let all_states: std::collections::HashMap<BranchParent, &S> = self
            .results
            .iter()
            .flat_map(|(history, _)| {
                history.program_i.iter().enumerate().map(|(idx, i)| {
                    (
                        BranchParent {
                            id: history.id,
                            idx,
                        },
                        &i.state,
                    )
                })
            })
            .collect();

        for (idx, instruction) in self.program.iter().enumerate() {
            let escaped_instruction = format!("{:?}", instruction)
                .replace("\\", "\\\\")
                .replace("\"", "'")
                .replace("\n", "\\n");
            write!(
                f,
                "subgraph cluster_{} {{ label=\"{}\"",
                idx, escaped_instruction
            )?;

            if self.unknown_operations.contains(&idx) {
                f.write_str(" style=dashed")?;
            }

            f.write_str(";\n")?;

            for reference in instruction_uses.entry(idx).or_default().iter() {
                write!(f, "\"b{}p{}\";", reference.id, reference.idx)?;
            }

            f.write_str("}\n")?;
        }

        f.write_str("};\n")?; //subgraph operations

        f.write_str("subgraph branches {\n")?;

        for (result_idx, (history, result)) in self.results.iter().enumerate() {
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
            let color_name_root = color_names[result_idx % color_names.len()];
            let color_name_suffix = match (result_idx / color_names.len()) % 4 {
                0 => "1",
                1 => "4",
                2 => "3",
                3 => "2",
                _ => "",
            }; //colors are easily confused after color_names.len() * 2, and outright reused after color_names.len() * 4
            write!(
                f,
                "edge [colorscheme=x11 color={}{}];",
                color_name_root, color_name_suffix
            )?;

            if history.program_i.len() > 0 {
                let mut program_iter = history.program_i.iter().enumerate();

                if let Some((idx, program_i)) = program_iter.next() {
                    //draw edge from the origin of this branch
                    if let Some(BranchParent {
                        idx: parent_idx,
                        id: parent_id,
                    }) = history.parent
                    {
                        let state_diff = match all_states.get(&BranchParent {
                            idx: parent_idx,
                            id: parent_id,
                        }) {
                            Some(prev_state) => program_i
                                .state
                                .diff(prev_state)
                                .replace("\\", "\\\\")
                                .replace("\"", "'")
                                .replace("\n", "\\n"),
                            None => String::new(),
                        };

                        write!(
                            f,
                            "\"b{}p{}\"-> \"b{}p{}\" [label=\"{}\"];\n",
                            parent_id, parent_idx, history.id, idx, state_diff
                        )?;
                    }
                    //draw edges for each of the operations
                    let mut prev_idx = idx;
                    let mut prev_state = &program_i.state;
                    while let Some((idx, program_i)) = program_iter.next() {
                        let state_diff = program_i
                            .state
                            .diff(prev_state)
                            .replace("\\", "\\\\")
                            .replace("\"", "'")
                            .replace("\n", "\\n");
                        write!(
                            f,
                            "\"b{}p{}\"-> \"b{}p{}\" [label=\"{}\"]\n",
                            history.id, prev_idx, history.id, idx, state_diff
                        )?;
                        prev_idx = idx;
                        prev_state = &program_i.state;
                    }
                }

                //draw edge to the result of this branch
                if history.program_i.len() > 0 {
                    let idx = history.program_i.len() - 1;
                    if let BranchResult::Dedup(BranchParent {
                        id: dedup_id,
                        idx: dedup_idx,
                    }) = result
                    {
                        write!(
                            f,
                            "\"b{}p{}\"->\"b{}p{}\" [style=dotted]",
                            history.id, idx, dedup_id, dedup_idx
                        )?;
                    } else {
                        let escaped_result = format!("{:?}", result)
                            .replace("\\", "\\\\")
                            .replace("\"", "'")
                            .replace("\n", "\\n");
                        write!(
                            f,
                            "\"b{}p{}\" ->\"{}\"; \"{}\" [shape=box];",
                            history.id, idx, escaped_result, escaped_result
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

impl<'q, R: Debug, S: Debug + DebugDiff, P: Debug> QueryPlanLogger<'q, R, S, P> {
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

    pub fn add_result(&mut self, history: BranchHistory<S>, result: BranchResult<R>) {
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

impl<'q, R: Debug, S: Debug + DebugDiff, P: Debug> Drop for QueryPlanLogger<'q, R, S, P> {
    fn drop(&mut self) {
        self.finish();
    }
}
