use crate::error::Error;
use crate::from_row::FromRow;
use crate::sqlite::connection::{execute, ConnectionState};
use crate::sqlite::type_info::DataType;
use crate::sqlite::SqliteTypeInfo;
use crate::HashMap;
use std::str::from_utf8;

// affinity
const SQLITE_AFF_NONE: u8 = 0x40; /* '@' */
const SQLITE_AFF_BLOB: u8 = 0x41; /* 'A' */
const SQLITE_AFF_TEXT: u8 = 0x42; /* 'B' */
const SQLITE_AFF_NUMERIC: u8 = 0x43; /* 'C' */
const SQLITE_AFF_INTEGER: u8 = 0x44; /* 'D' */
const SQLITE_AFF_REAL: u8 = 0x45; /* 'E' */

// opcodes
const OP_INIT: &str = "Init";
const OP_GOTO: &str = "Goto";
const OP_DECR_JUMP_ZERO: &str = "DecrJumpZero";
const OP_ELSE_EQ: &str = "ElseEq";
const OP_EQ: &str = "Eq";
const OP_END_COROUTINE: &str = "EndCoroutine";
const OP_FILTER: &str = "Filter";
const OP_FK_IF_ZERO: &str = "FkIfZero";
const OP_FOUND: &str = "Found";
const OP_GE: &str = "Ge";
const OP_GO_SUB: &str = "Gosub";
const OP_GT: &str = "Gt";
const OP_IDX_GE: &str = "IdxGE";
const OP_IDX_GT: &str = "IdxGT";
const OP_IDX_LE: &str = "IdxLE";
const OP_IDX_LT: &str = "IdxLT";
const OP_IF: &str = "If";
const OP_IF_NO_HOPE: &str = "IfNoHope";
const OP_IF_NOT: &str = "IfNot";
const OP_IF_NOT_OPEN: &str = "IfNotOpen";
const OP_IF_NOT_ZERO: &str = "IfNotZero";
const OP_IF_NULL_ROW: &str = "IfNullRow";
const OP_IF_POS: &str = "IfPos";
const OP_IF_SMALLER: &str = "IfSmaller";
const OP_INCR_VACUUM: &str = "IncrVacuum";
const OP_INIT_COROUTINE: &str = "InitCoroutine";
const OP_IS_NULL: &str = "IsNull";
const OP_IS_NULL_OR_TYPE: &str = "IsNullOrType";
const OP_LAST: &str = "Last";
const OP_LE: &str = "Le";
const OP_LT: &str = "Lt";
const OP_MUST_BE_INT: &str = "MustBeInt";
const OP_NE: &str = "Ne";
const OP_NEXT: &str = "Next";
const OP_NO_CONFLICT: &str = "NoConflict";
const OP_NOT_EXISTS: &str = "NotExists";
const OP_NOT_NULL: &str = "NotNull";
const OP_ONCE: &str = "Once";
const OP_PREV: &str = "Prev";
const OP_PROGRAM: &str = "Program";
const OP_RETURN: &str = "Return";
const OP_REWIND: &str = "Rewind";
const OP_ROW_DATA: &str = "RowData";
const OP_ROW_SET_READ: &str = "RowSetRead";
const OP_ROW_SET_TEST: &str = "RowSetTest";
const OP_SEEK_GE: &str = "SeekGE";
const OP_SEEK_GT: &str = "SeekGT";
const OP_SEEK_LE: &str = "SeekLE";
const OP_SEEK_LT: &str = "SeekLT";
const OP_SEEK_ROW_ID: &str = "SeekRowId";
const OP_SEEK_SCAN: &str = "SeekScan";
const OP_SEQUENCE_TEST: &str = "SequenceTest";
const OP_SORTER_NEXT: &str = "SorterNext";
const OP_SORTER_SORT: &str = "SorterSort";
const OP_V_FILTER: &str = "VFilter";
const OP_V_NEXT: &str = "VNext";
const OP_YIELD: &str = "Yield";
const OP_JUMP: &str = "Jump";
const OP_COLUMN: &str = "Column";
const OP_MAKE_RECORD: &str = "MakeRecord";
const OP_INSERT: &str = "Insert";
const OP_IDX_INSERT: &str = "IdxInsert";
const OP_OPEN_PSEUDO: &str = "OpenPseudo";
const OP_OPEN_READ: &str = "OpenRead";
const OP_OPEN_WRITE: &str = "OpenWrite";
const OP_OPEN_EPHEMERAL: &str = "OpenEphemeral";
const OP_OPEN_AUTOINDEX: &str = "OpenAutoindex";
const OP_AGG_FINAL: &str = "AggFinal";
const OP_AGG_STEP: &str = "AggStep";
const OP_FUNCTION: &str = "Function";
const OP_MOVE: &str = "Move";
const OP_COPY: &str = "Copy";
const OP_SCOPY: &str = "SCopy";
const OP_NULL: &str = "Null";
const OP_NULL_ROW: &str = "NullRow";
const OP_INT_COPY: &str = "IntCopy";
const OP_CAST: &str = "Cast";
const OP_STRING8: &str = "String8";
const OP_INT64: &str = "Int64";
const OP_INTEGER: &str = "Integer";
const OP_REAL: &str = "Real";
const OP_NOT: &str = "Not";
const OP_BLOB: &str = "Blob";
const OP_VARIABLE: &str = "Variable";
const OP_COUNT: &str = "Count";
const OP_ROWID: &str = "Rowid";
const OP_NEWROWID: &str = "NewRowid";
const OP_OR: &str = "Or";
const OP_AND: &str = "And";
const OP_BIT_AND: &str = "BitAnd";
const OP_BIT_OR: &str = "BitOr";
const OP_SHIFT_LEFT: &str = "ShiftLeft";
const OP_SHIFT_RIGHT: &str = "ShiftRight";
const OP_ADD: &str = "Add";
const OP_SUBTRACT: &str = "Subtract";
const OP_MULTIPLY: &str = "Multiply";
const OP_DIVIDE: &str = "Divide";
const OP_REMAINDER: &str = "Remainder";
const OP_CONCAT: &str = "Concat";
const OP_RESULT_ROW: &str = "ResultRow";
const OP_HALT: &str = "Halt";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ColumnType {
    pub datatype: DataType,
    pub nullable: Option<bool>,
}

impl Default for ColumnType {
    fn default() -> Self {
        Self {
            datatype: DataType::Null,
            nullable: None,
        }
    }
}

impl ColumnType {
    fn null() -> Self {
        Self {
            datatype: DataType::Null,
            nullable: Some(true),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum RegDataType {
    Single(ColumnType),
    Record(Vec<ColumnType>),
    Int(i64),
}

impl RegDataType {
    fn map_to_datatype(&self) -> DataType {
        match self {
            RegDataType::Single(d) => d.datatype,
            RegDataType::Record(_) => DataType::Null, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
            RegDataType::Int(_) => DataType::Int,
        }
    }
    fn map_to_nullable(&self) -> Option<bool> {
        match self {
            RegDataType::Single(d) => d.nullable,
            RegDataType::Record(_) => None, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
            RegDataType::Int(_) => Some(false),
        }
    }
    fn map_to_columntype(&self) -> ColumnType {
        match self {
            RegDataType::Single(d) => *d,
            RegDataType::Record(_) => ColumnType {
                datatype: DataType::Null,
                nullable: None,
            }, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
            RegDataType::Int(_) => ColumnType {
                datatype: DataType::Int,
                nullable: Some(false),
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CursorDataType {
    Normal(HashMap<i64, ColumnType>),
    Pseudo(i64),
}

impl CursorDataType {
    fn from_sparse_record(record: &HashMap<i64, ColumnType>) -> Self {
        Self::Normal(
            record
                .iter()
                .map(|(&colnum, &datatype)| (colnum, datatype))
                .collect(),
        )
    }

    fn from_dense_record(record: &Vec<ColumnType>) -> Self {
        Self::Normal((0..).zip(record.iter().copied()).collect())
    }

    fn map_to_dense_record(&self, registers: &HashMap<i64, RegDataType>) -> Vec<ColumnType> {
        match self {
            Self::Normal(record) => {
                let mut rowdata = vec![ColumnType::default(); record.len()];
                for (idx, col) in record.iter() {
                    rowdata[*idx as usize] = col.clone();
                }
                rowdata
            }
            Self::Pseudo(i) => match registers.get(i) {
                Some(RegDataType::Record(r)) => r.clone(),
                _ => Vec::new(),
            },
        }
    }

    fn map_to_sparse_record(
        &self,
        registers: &HashMap<i64, RegDataType>,
    ) -> HashMap<i64, ColumnType> {
        match self {
            Self::Normal(c) => c.clone(),
            Self::Pseudo(i) => match registers.get(i) {
                Some(RegDataType::Record(r)) => (0..).zip(r.iter().copied()).collect(),
                _ => HashMap::new(),
            },
        }
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn affinity_to_type(affinity: u8) -> DataType {
    match affinity {
        SQLITE_AFF_BLOB => DataType::Blob,
        SQLITE_AFF_INTEGER => DataType::Int64,
        SQLITE_AFF_NUMERIC => DataType::Numeric,
        SQLITE_AFF_REAL => DataType::Float,
        SQLITE_AFF_TEXT => DataType::Text,

        SQLITE_AFF_NONE | _ => DataType::Null,
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn opcode_to_type(op: &str) -> DataType {
    match op {
        OP_REAL => DataType::Float,
        OP_BLOB => DataType::Blob,
        OP_AND | OP_OR => DataType::Bool,
        OP_ROWID | OP_COUNT | OP_INT64 | OP_INTEGER => DataType::Int64,
        OP_STRING8 => DataType::Text,
        OP_COLUMN | _ => DataType::Null,
    }
}

fn root_block_columns(
    conn: &mut ConnectionState,
) -> Result<HashMap<i64, HashMap<i64, ColumnType>>, Error> {
    let table_block_columns: Vec<(i64, i64, String, bool)> = execute::iter(
        conn,
        "SELECT s.rootpage, col.cid as colnum, col.type, col.\"notnull\"
         FROM (select * from sqlite_temp_schema UNION select * from sqlite_schema) s
         JOIN pragma_table_info(s.name) AS col
         WHERE s.type = 'table'",
        None,
        false,
    )?
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(&row?))
    .collect::<Result<Vec<_>, Error>>()?;

    let index_block_columns: Vec<(i64, i64, String, bool)> = execute::iter(
        conn,
        "SELECT s.rootpage, idx.seqno as colnum, col.type, col.\"notnull\"
         FROM (select * from sqlite_temp_schema UNION select * from sqlite_schema) s
         JOIN pragma_index_info(s.name) AS idx
         LEFT JOIN pragma_table_info(s.tbl_name) as col
           ON col.cid = idx.cid
           WHERE s.type = 'index'",
        None,
        false,
    )?
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(&row?))
    .collect::<Result<Vec<_>, Error>>()?;

    let mut row_info: HashMap<i64, HashMap<i64, ColumnType>> = HashMap::new();
    for (block, colnum, datatype, notnull) in table_block_columns {
        let row_info = row_info.entry(block).or_default();
        row_info.insert(
            colnum,
            ColumnType {
                datatype: datatype.parse().unwrap_or(DataType::Null),
                nullable: Some(!notnull),
            },
        );
    }
    for (block, colnum, datatype, notnull) in index_block_columns {
        let row_info = row_info.entry(block).or_default();
        row_info.insert(
            colnum,
            ColumnType {
                datatype: datatype.parse().unwrap_or(DataType::Null),
                nullable: Some(!notnull),
            },
        );
    }

    return Ok(row_info);
}

#[derive(Debug, Clone, PartialEq)]
struct QueryState {
    pub visited: Vec<bool>,
    pub history: Vec<usize>,
    // Registers
    pub r: HashMap<i64, RegDataType>,
    // Rows that pointers point to
    pub p: HashMap<i64, CursorDataType>,
    // Next instruction to execute
    pub program_i: usize,
    // Results published by the execution
    pub result: Option<Vec<(Option<SqliteTypeInfo>, Option<bool>)>>,
}

// Opcode Reference: https://sqlite.org/opcode.html
pub(super) fn explain(
    conn: &mut ConnectionState,
    query: &str,
) -> Result<(Vec<SqliteTypeInfo>, Vec<Option<bool>>), Error> {
    let root_block_cols = root_block_columns(conn)?;
    let program: Vec<(i64, String, i64, i64, i64, Vec<u8>)> =
        execute::iter(conn, &format!("EXPLAIN {}", query), None, false)?
            .filter_map(|res| res.map(|either| either.right()).transpose())
            .map(|row| FromRow::from_row(&row?))
            .collect::<Result<Vec<_>, Error>>()?;
    let program_size = program.len();

    let mut logger =
        crate::logger::QueryPlanLogger::new(query, &program, conn.log_settings.clone());

    let mut states = vec![QueryState {
        visited: vec![false; program_size],
        history: Vec::new(),
        r: HashMap::with_capacity(6),
        p: HashMap::with_capacity(6),
        program_i: 0,
        result: None,
    }];

    let mut result_states = Vec::new();

    while let Some(mut state) = states.pop() {
        while state.program_i < program_size {
            if state.visited[state.program_i] {
                state.program_i += 1;
                //avoid (infinite) loops by breaking if we ever hit the same instruction twice
                break;
            }
            let (_, ref opcode, p1, p2, p3, ref p4) = program[state.program_i];
            state.history.push(state.program_i);

            match &**opcode {
                OP_INIT => {
                    // start at <p2>
                    state.visited[state.program_i] = true;
                    state.program_i = p2 as usize;
                    continue;
                }

                OP_GOTO => {
                    // goto <p2>
                    state.visited[state.program_i] = true;
                    state.program_i = p2 as usize;
                    continue;
                }

                OP_DECR_JUMP_ZERO | OP_ELSE_EQ | OP_EQ | OP_FILTER | OP_FK_IF_ZERO | OP_FOUND
                | OP_GE | OP_GO_SUB | OP_GT | OP_IDX_GE | OP_IDX_GT | OP_IDX_LE | OP_IDX_LT
                | OP_IF | OP_IF_NO_HOPE | OP_IF_NOT | OP_IF_NOT_OPEN | OP_IF_NOT_ZERO
                | OP_IF_NULL_ROW | OP_IF_POS | OP_IF_SMALLER | OP_INCR_VACUUM | OP_IS_NULL
                | OP_IS_NULL_OR_TYPE | OP_LE | OP_LAST | OP_LT | OP_MUST_BE_INT | OP_NE
                | OP_NEXT | OP_NO_CONFLICT | OP_NOT_EXISTS | OP_NOT_NULL | OP_ONCE | OP_PREV
                | OP_PROGRAM | OP_ROW_SET_READ | OP_ROW_SET_TEST | OP_SEEK_GE | OP_SEEK_GT
                | OP_SEEK_LE | OP_SEEK_LT | OP_SEEK_ROW_ID | OP_SEEK_SCAN | OP_SEQUENCE_TEST
                | OP_SORTER_NEXT | OP_SORTER_SORT | OP_V_FILTER | OP_V_NEXT | OP_REWIND => {
                    // goto <p2> or next instruction (depending on actual values)
                    state.visited[state.program_i] = true;

                    let mut branch_state = state.clone();
                    branch_state.program_i = p2 as usize;
                    states.push(branch_state);

                    state.program_i += 1;
                    continue;
                }

                OP_INIT_COROUTINE => {
                    // goto <p2> or next instruction (depending on actual values)
                    state.visited[state.program_i] = true;
                    state.r.insert(p1, RegDataType::Int(p3));

                    if p2 != 0 {
                        state.program_i = p2 as usize;
                    } else {
                        state.program_i += 1;
                    }
                    continue;
                }

                OP_END_COROUTINE => {
                    // jump to p2 of the yield instruction pointed at by register p1
                    state.visited[state.program_i] = true;
                    if let Some(RegDataType::Int(yield_i)) = state.r.get(&p1) {
                        if let Some((_, yield_op, _, yield_p2, _, _)) =
                            program.get(*yield_i as usize)
                        {
                            if OP_YIELD == yield_op.as_str() {
                                state.program_i = (*yield_p2) as usize;
                                state.r.remove(&p1);
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                OP_RETURN => {
                    // jump to the instruction after the instruction pointed at by register p1
                    state.visited[state.program_i] = true;
                    if let Some(RegDataType::Int(return_i)) = state.r.get(&p1) {
                        state.program_i = (*return_i + 1) as usize;
                        state.r.remove(&p1);
                        continue;
                    } else {
                        break;
                    }
                }

                OP_YIELD => {
                    // jump to p2 of the yield instruction pointed at by register p1, store prior instruction in p1
                    state.visited[state.program_i] = true;
                    if let Some(RegDataType::Int(yield_i)) = state.r.get_mut(&p1) {
                        let program_i: usize = state.program_i;

                        //if yielding to a yield operation, go to the NEXT instruction after that instruction
                        if program
                            .get(*yield_i as usize)
                            .map(|(_, yield_op, _, _, _, _)| yield_op.as_str())
                            == Some(OP_YIELD)
                        {
                            state.program_i = (*yield_i + 1) as usize;
                            *yield_i = program_i as i64;
                            continue;
                        } else {
                            state.program_i = *yield_i as usize;
                            *yield_i = program_i as i64;
                            continue;
                        }
                    } else {
                        break;
                    }
                }

                OP_JUMP => {
                    // goto one of <p1>, <p2>, or <p3> based on the result of a prior compare
                    state.visited[state.program_i] = true;

                    let mut branch_state = state.clone();
                    branch_state.program_i = p1 as usize;
                    states.push(branch_state);

                    let mut branch_state = state.clone();
                    branch_state.program_i = p2 as usize;
                    states.push(branch_state);

                    let mut branch_state = state.clone();
                    branch_state.program_i = p3 as usize;
                    states.push(branch_state);
                }

                OP_COLUMN => {
                    //Get the row stored at p1, or NULL; get the column stored at p2, or NULL
                    if let Some(record) = state.p.get(&p1).map(|c| c.map_to_sparse_record(&state.r))
                    {
                        if let Some(col) = record.get(&p2) {
                            // insert into p3 the datatype of the col
                            state.r.insert(p3, RegDataType::Single(*col));
                        } else {
                            state
                                .r
                                .insert(p3, RegDataType::Single(ColumnType::default()));
                        }
                    } else {
                        state
                            .r
                            .insert(p3, RegDataType::Single(ColumnType::default()));
                    }
                }

                OP_ROW_DATA => {
                    //Get entire row from cursor p1, store it into register p2
                    if let Some(record) = state.p.get(&p1) {
                        let rowdata = record.map_to_dense_record(&state.r);
                        state.r.insert(p2, RegDataType::Record(rowdata));
                    } else {
                        state.r.insert(p2, RegDataType::Record(Vec::new()));
                    }
                }

                OP_MAKE_RECORD => {
                    // p3 = Record([p1 .. p1 + p2])
                    let mut record = Vec::with_capacity(p2 as usize);
                    for reg in p1..p1 + p2 {
                        record.push(
                            state
                                .r
                                .get(&reg)
                                .map(|d| d.clone().map_to_columntype())
                                .unwrap_or(ColumnType::default()),
                        );
                    }
                    state.r.insert(p3, RegDataType::Record(record));
                }

                OP_INSERT | OP_IDX_INSERT => {
                    if let Some(RegDataType::Record(record)) = state.r.get(&p2) {
                        if let Some(CursorDataType::Normal(row)) = state.p.get_mut(&p1) {
                            // Insert the record into wherever pointer p1 is
                            *row = (0..).zip(record.iter().copied()).collect();
                        }
                    }
                    //Noop if the register p2 isn't a record, or if pointer p1 does not exist
                }

                OP_OPEN_PSEUDO => {
                    // Create a cursor p1 aliasing the record from register p2
                    state.p.insert(p1, CursorDataType::Pseudo(p2));
                }
                OP_OPEN_READ | OP_OPEN_WRITE => {
                    //Create a new pointer which is referenced by p1, take column metadata from db schema if found
                    if p3 == 0 {
                        if let Some(columns) = root_block_cols.get(&p2) {
                            state
                                .p
                                .insert(p1, CursorDataType::from_sparse_record(columns));
                        } else {
                            state
                                .p
                                .insert(p1, CursorDataType::Normal(HashMap::with_capacity(6)));
                        }
                    } else {
                        state
                            .p
                            .insert(p1, CursorDataType::Normal(HashMap::with_capacity(6)));
                    }
                }

                OP_OPEN_EPHEMERAL | OP_OPEN_AUTOINDEX => {
                    //Create a new pointer which is referenced by p1
                    state.p.insert(
                        p1,
                        CursorDataType::from_dense_record(&vec![ColumnType::null(); p2 as usize]),
                    );
                }

                OP_VARIABLE => {
                    // r[p2] = <value of variable>
                    state.r.insert(p2, RegDataType::Single(ColumnType::null()));
                }

                OP_FUNCTION => {
                    // r[p1] = func( _ )
                    match from_utf8(p4).map_err(Error::protocol)? {
                        "last_insert_rowid(0)" => {
                            // last_insert_rowid() -> INTEGER
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType {
                                    datatype: DataType::Int64,
                                    nullable: Some(false),
                                }),
                            );
                        }

                        _ => logger.add_unknown_operation(&program[state.program_i]),
                    }
                }

                OP_NULL_ROW => {
                    // all columns in cursor X are potentially nullable
                    if let Some(CursorDataType::Normal(ref mut cursor)) = state.p.get_mut(&p1) {
                        for ref mut col in cursor.values_mut() {
                            col.nullable = Some(true);
                        }
                    }
                    //else we don't know about the cursor
                }

                OP_AGG_STEP => {
                    //assume that AGG_FINAL will be called
                    let p4 = from_utf8(p4).map_err(Error::protocol)?;

                    if p4.starts_with("count(") {
                        // count(_) -> INTEGER
                        state.r.insert(
                            p3,
                            RegDataType::Single(ColumnType {
                                datatype: DataType::Int64,
                                nullable: Some(false),
                            }),
                        );
                    } else if let Some(v) = state.r.get(&p2).cloned() {
                        // r[p3] = AGG ( r[p2] )
                        state.r.insert(p3, v);
                    }
                }

                OP_AGG_FINAL => {
                    let p4 = from_utf8(p4).map_err(Error::protocol)?;

                    if p4.starts_with("count(") {
                        // count(_) -> INTEGER
                        state.r.insert(
                            p1,
                            RegDataType::Single(ColumnType {
                                datatype: DataType::Int64,
                                nullable: Some(false),
                            }),
                        );
                    } else if let Some(v) = state.r.get(&p2).cloned() {
                        // r[p3] = AGG ( r[p2] )
                        state.r.insert(p3, v);
                    }
                }

                OP_CAST => {
                    // affinity(r[p1])
                    if let Some(v) = state.r.get_mut(&p1) {
                        *v = RegDataType::Single(ColumnType {
                            datatype: affinity_to_type(p2 as u8),
                            nullable: v.map_to_nullable(),
                        });
                    }
                }

                OP_COPY | OP_MOVE | OP_SCOPY | OP_INT_COPY => {
                    // r[p2] = r[p1]
                    if let Some(v) = state.r.get(&p1).cloned() {
                        state.r.insert(p2, v);
                    }
                }

                OP_INTEGER => {
                    // r[p2] = p1
                    state.r.insert(p2, RegDataType::Int(p1));
                }

                OP_BLOB | OP_COUNT | OP_REAL | OP_STRING8 | OP_ROWID | OP_NEWROWID => {
                    // r[p2] = <value of constant>
                    state.r.insert(
                        p2,
                        RegDataType::Single(ColumnType {
                            datatype: opcode_to_type(&opcode),
                            nullable: Some(false),
                        }),
                    );
                }

                OP_NOT => {
                    // r[p2] = NOT r[p1]
                    if let Some(a) = state.r.get(&p1).cloned() {
                        state.r.insert(p2, a);
                    }
                }

                OP_NULL => {
                    // r[p2..p3] = null
                    let idx_range = if p2 < p3 { p2..=p3 } else { p2..=p2 };

                    for idx in idx_range {
                        state.r.insert(idx, RegDataType::Single(ColumnType::null()));
                    }
                }

                OP_OR | OP_AND | OP_BIT_AND | OP_BIT_OR | OP_SHIFT_LEFT | OP_SHIFT_RIGHT
                | OP_ADD | OP_SUBTRACT | OP_MULTIPLY | OP_DIVIDE | OP_REMAINDER | OP_CONCAT => {
                    // r[p3] = r[p1] + r[p2]
                    match (state.r.get(&p1).cloned(), state.r.get(&p2).cloned()) {
                        (Some(a), Some(b)) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType {
                                    datatype: if matches!(a.map_to_datatype(), DataType::Null) {
                                        b.map_to_datatype()
                                    } else {
                                        a.map_to_datatype()
                                    },
                                    nullable: match (a.map_to_nullable(), b.map_to_nullable()) {
                                        (Some(a_n), Some(b_n)) => Some(a_n | b_n),
                                        (Some(a_n), None) => Some(a_n),
                                        (None, Some(b_n)) => Some(b_n),
                                        (None, None) => None,
                                    },
                                }),
                            );
                        }

                        (Some(v), None) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType {
                                    datatype: v.map_to_datatype(),
                                    nullable: None,
                                }),
                            );
                        }

                        (None, Some(v)) => {
                            state.r.insert(
                                p3,
                                RegDataType::Single(ColumnType {
                                    datatype: v.map_to_datatype(),
                                    nullable: None,
                                }),
                            );
                        }

                        _ => {}
                    }
                }

                OP_RESULT_ROW => {
                    // output = r[p1 .. p1 + p2]
                    state.visited[state.program_i] = true;
                    state.result = Some(
                        (p1..p1 + p2)
                            .map(|i| {
                                let coltype = state.r.get(&i);

                                let sqltype =
                                    coltype.map(|d| d.map_to_datatype()).map(SqliteTypeInfo);
                                let nullable =
                                    coltype.map(|d| d.map_to_nullable()).unwrap_or_default();

                                (sqltype, nullable)
                            })
                            .collect(),
                    );

                    if logger.log_enabled() {
                        let program_history: Vec<&(i64, String, i64, i64, i64, Vec<u8>)> =
                            state.history.iter().map(|i| &program[*i]).collect();
                        logger.add_result((program_history, state.result.clone()));
                    }

                    result_states.push(state.clone());
                }

                OP_HALT => {
                    break;
                }

                _ => {
                    // ignore unsupported operations
                    // if we fail to find an r later, we just give up
                    logger.add_unknown_operation(&program[state.program_i]);
                }
            }

            state.visited[state.program_i] = true;
            state.program_i += 1;
        }
    }

    let mut output: Vec<Option<SqliteTypeInfo>> = Vec::new();
    let mut nullable: Vec<Option<bool>> = Vec::new();

    while let Some(state) = result_states.pop() {
        // find the datatype info from each ResultRow execution
        if let Some(result) = state.result {
            let mut idx = 0;
            for (this_type, this_nullable) in result {
                if output.len() == idx {
                    output.push(this_type);
                } else if output[idx].is_none()
                    || matches!(output[idx], Some(SqliteTypeInfo(DataType::Null)))
                {
                    output[idx] = this_type;
                }

                if nullable.len() == idx {
                    nullable.push(this_nullable);
                } else if let Some(ref mut null) = nullable[idx] {
                    //if any ResultRow's column is nullable, the final result is nullable
                    if let Some(this_null) = this_nullable {
                        *null |= this_null;
                    }
                } else {
                    nullable[idx] = this_nullable;
                }
                idx += 1;
            }
        }
    }

    let output = output
        .into_iter()
        .map(|o| o.unwrap_or(SqliteTypeInfo(DataType::Null)))
        .collect();

    Ok((output, nullable))
}

#[test]
fn test_root_block_columns_has_types() {
    use crate::sqlite::SqliteConnectOptions;
    use std::str::FromStr;
    let conn_options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
    let mut conn = super::EstablishParams::from_options(&conn_options)
        .unwrap()
        .establish()
        .unwrap();

    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t(a INTEGER PRIMARY KEY, b_null TEXT NULL, b TEXT NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(
        execute::iter(&mut conn, r"CREATE INDEX i1 on t (a,b_null);", None, false)
            .unwrap()
            .next()
            .is_some()
    );
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX i2 on t (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t2(a INTEGER NOT NULL, b_null NUMERIC NULL, b NUMERIC NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE INDEX t2i1 on t2 (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX t2i2 on t2 (a,b);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());

    let table_block_nums: HashMap<String, i64> = execute::iter(
        &mut conn,
        r"select name, rootpage from sqlite_master",
        None,
        false,
    )
    .unwrap()
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(row.as_ref().unwrap()))
    .collect::<Result<HashMap<_, _>, Error>>()
    .unwrap();

    let root_block_cols = root_block_columns(&mut conn).unwrap();

    assert_eq!(6, root_block_cols.len());

    //prove that we have some information for each table & index
    for blocknum in table_block_nums.values() {
        assert!(root_block_cols.contains_key(blocknum));
    }

    //prove that each block has the correct information
    {
        let blocknum = table_block_nums["t"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&blocknum][&1]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Text,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&2]
        );
    }

    {
        let blocknum = table_block_nums["i1"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&blocknum][&1]
        );
    }

    {
        let blocknum = table_block_nums["i2"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(true) //sqlite primary key columns are nullable unless declared not null
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Text,
                nullable: Some(true)
            },
            root_block_cols[&blocknum][&1]
        );
    }

    {
        let blocknum = table_block_nums["t2"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Null,
                nullable: Some(true)
            },
            root_block_cols[&blocknum][&1]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Null,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&2]
        );
    }

    {
        let blocknum = table_block_nums["t2i1"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Null,
                nullable: Some(true)
            },
            root_block_cols[&blocknum][&1]
        );
    }

    {
        let blocknum = table_block_nums["t2i2"];
        assert_eq!(
            ColumnType {
                datatype: DataType::Int64,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&0]
        );
        assert_eq!(
            ColumnType {
                datatype: DataType::Null,
                nullable: Some(false)
            },
            root_block_cols[&blocknum][&1]
        );
    }
}
