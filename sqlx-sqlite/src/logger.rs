use crate::connection::intmap::IntMap;
use sqlx_core::{connection::LogSettings, logger};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

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
    pub id: i64,
    pub idx: i64,
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
    branch_origins: IntMap<BranchParent>,
    branch_results: IntMap<BranchResult<R>>,
    branch_operations: IntMap<IntMap<InstructionHistory<S>>>,
    program: &'q [P],
    settings: LogSettings,
}

impl<R: Debug, S: Debug + DebugDiff, P: Debug> core::fmt::Display for QueryPlanLogger<'_, R, S, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //writes query plan history in dot format
        f.write_str("digraph {\n")?;

        f.write_str("subgraph operations {\n")?;
        f.write_str("style=\"rounded\";\nnode [shape=\"point\"];\n")?;

        let all_states: std::collections::HashMap<BranchParent, &InstructionHistory<S>> = self
            .branch_operations
            .iter_entries()
            .flat_map(
                |(branch_id, instructions): (i64, &IntMap<InstructionHistory<S>>)| {
                    instructions.iter_entries().map(
                        move |(idx, ih): (i64, &InstructionHistory<S>)| {
                            (BranchParent { id: branch_id, idx }, ih)
                        },
                    )
                },
            )
            .collect();

        //using BTreeMap for predictable ordering
        let mut instruction_uses: IntMap<Vec<BranchParent>> = Default::default();
        for (k, state) in all_states.iter() {
            let entry = instruction_uses.get_mut_or_default(&(state.program_i as i64));
            entry.push(k.clone());
        }

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

            for reference in instruction_uses
                .get(&(idx as i64))
                .unwrap_or(&Vec::new())
                .iter()
            {
                write!(f, "\"b{}p{}\";", reference.id, reference.idx)?;
            }

            f.write_str("}\n")?;
        }

        f.write_str("};\n")?; //subgraph operations

        let max_branch_id: i64 = [
            self.branch_operations.last_index().unwrap_or(0),
            self.branch_results.last_index().unwrap_or(0),
            self.branch_results.last_index().unwrap_or(0),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);

        f.write_str("subgraph branches {\n")?;
        for branch_id in 0..=max_branch_id {
            write!(f, "subgraph b{}{{", branch_id)?;

            let branch_num = branch_id as usize;
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
            let color_name_root = color_names[branch_num % color_names.len()];
            let color_name_suffix = match (branch_num / color_names.len()) % 4 {
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

            let mut instruction_list: Vec<(BranchParent, &InstructionHistory<S>)> = Vec::new();
            if let Some(parent) = self.branch_origins.get(&branch_id) {
                if let Some(parent_state) = all_states.get(parent) {
                    instruction_list.push((parent.clone(), parent_state));
                } else {
                    dbg!("no state for parent", parent);
                }
            }
            if let Some(instructions) = self.branch_operations.get(&branch_id) {
                for instruction in instructions.iter_entries() {
                    instruction_list.push((
                        BranchParent {
                            id: branch_id,
                            idx: instruction.0,
                        },
                        instruction.1,
                    ))
                }
            }

            let mut instructions_iter = instruction_list.into_iter();

            if let Some((cur_ref, cur_instruction)) = instructions_iter.next() {
                let mut prev_ref = cur_ref;
                let mut prev_instruction = cur_instruction;

                while let Some((cur_ref, cur_instruction)) = instructions_iter.next() {
                    let state_diff = cur_instruction
                        .state
                        .diff(&prev_instruction.state)
                        .replace("\\", "\\\\")
                        .replace("\"", "'")
                        .replace("\n", "\\n");
                    write!(
                        f,
                        "\"b{}p{}\"-> \"b{}p{}\" [label=\"{}\"]\n",
                        prev_ref.id, prev_ref.idx, cur_ref.id, cur_ref.idx, state_diff
                    )?;

                    prev_ref = cur_ref;
                    prev_instruction = cur_instruction;
                }

                //draw edge to the result of this branch
                if let Some(result) = self.branch_results.get(&branch_id) {
                    if let BranchResult::Dedup(dedup_ref) = result {
                        write!(
                            f,
                            "\"b{}p{}\"->\"b{}p{}\" [style=dotted]",
                            prev_ref.id, prev_ref.idx, dedup_ref.id, dedup_ref.idx
                        )?;
                    } else {
                        let escaped_result = format!("{:?}", result)
                            .replace("\\", "\\\\")
                            .replace("\"", "'")
                            .replace("\n", "\\n");
                        write!(
                            f,
                            "\"b{}p{}\" ->\"{}\"; \"{}\" [shape=box];",
                            prev_ref.id, prev_ref.idx, escaped_result, escaped_result
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
            branch_origins: IntMap::new(),
            branch_results: IntMap::new(),
            branch_operations: IntMap::new(),
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

    pub fn add_branch<I: Copy>(&mut self, state: I, parent: &BranchParent)
    where
        BranchParent: From<I>,
    {
        let branch: BranchParent = BranchParent::from(state);
        self.branch_origins.insert(branch.id, parent.clone());
    }

    pub fn add_operation<I: Copy>(&mut self, program_i: usize, state: I)
    where
        BranchParent: From<I>,
        S: From<I>,
    {
        let branch: BranchParent = BranchParent::from(state);
        let state: S = S::from(state);
        self.branch_operations
            .get_mut_or_default(&branch.id)
            .insert(branch.idx, InstructionHistory { program_i, state });
    }

    pub fn add_result<I>(&mut self, state: I, result: BranchResult<R>)
    where
        BranchParent: for<'a> From<&'a I>,
        S: From<I>,
    {
        let branch: BranchParent = BranchParent::from(&state);
        self.branch_results.insert(branch.id, result);
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
