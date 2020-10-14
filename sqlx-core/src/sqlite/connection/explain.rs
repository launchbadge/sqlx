use crate::error::Error;
use crate::query_as::query_as;
use crate::sqlite::type_info::DataType;
use crate::sqlite::{SqliteConnection, SqliteTypeInfo};
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
const OP_COLUMN: &str = "Column";
const OP_AGG_STEP: &str = "AggStep";
const OP_FUNCTION: &str = "Function";
const OP_MOVE: &str = "Move";
const OP_COPY: &str = "Copy";
const OP_SCOPY: &str = "SCopy";
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

pub(super) async fn explain(
    conn: &mut SqliteConnection,
    query: &str,
) -> Result<(Vec<SqliteTypeInfo>, Vec<Option<bool>>), Error> {
    let mut r = HashMap::<i64, DataType>::with_capacity(6);
    let mut n = HashMap::<i64, bool>::with_capacity(6);

    let program =
        query_as::<_, (i64, String, i64, i64, i64, Vec<u8>)>(&*format!("EXPLAIN {}", query))
            .fetch_all(&mut *conn)
            .await?;

    let mut program_i = 0;
    let program_size = program.len();

    while program_i < program_size {
        let (_, ref opcode, p1, p2, p3, ref p4) = program[program_i];

        match &**opcode {
            OP_INIT => {
                // start at <p2>
                program_i = p2 as usize;
                continue;
            }

            OP_GOTO => {
                // goto <p2>
                program_i = p2 as usize;
                continue;
            }

            OP_COLUMN => {
                // r[p3] = <value of column>
                r.insert(p3, DataType::Null);
                n.insert(p3, true);
            }

            OP_VARIABLE => {
                // r[p2] = <value of variable>
                r.insert(p2, DataType::Null);
                n.insert(p3, true);
            }

            OP_FUNCTION => {
                // r[p1] = func( _ )
                match from_utf8(p4).map_err(Error::protocol)? {
                    "last_insert_rowid(0)" => {
                        // last_insert_rowid() -> INTEGER
                        r.insert(p3, DataType::Int64);
                        n.insert(p3, false);
                    }

                    _ => {}
                }
            }

            OP_AGG_STEP => {
                let p4 = from_utf8(p4).map_err(Error::protocol)?;

                if p4.starts_with("count(") {
                    // count(_) -> INTEGER
                    r.insert(p3, DataType::Int64);
                    n.insert(p3, false);
                } else if let Some(v) = r.get(&p2).copied() {
                    // r[p3] = AGG ( r[p2] )
                    r.insert(p3, v);
                    let val = n.get(&p2).copied().unwrap_or(true);
                    n.insert(p3, val);
                }
            }

            OP_CAST => {
                // affinity(r[p1])
                if let Some(v) = r.get_mut(&p1) {
                    *v = affinity_to_type(p2 as u8);
                }
            }

            OP_COPY | OP_MOVE | OP_SCOPY | OP_INT_COPY => {
                // r[p2] = r[p1]
                if let Some(v) = r.get(&p1).copied() {
                    r.insert(p2, v);
                    let val = n.get(&p1).copied().unwrap_or(true);
                    n.insert(p2, val);
                }
            }

            OP_OR | OP_AND | OP_BLOB | OP_COUNT | OP_REAL | OP_STRING8 | OP_INTEGER | OP_ROWID => {
                // r[p2] = <value of constant>
                r.insert(p2, opcode_to_type(&opcode));
                n.insert(p2, false);
            }

            OP_NOT => {
                // r[p2] = NOT r[p1]
                if let Some(a) = r.get(&p1).copied() {
                    r.insert(p2, a);
                    let val = n.get(&p1).copied().unwrap_or(true);
                    n.insert(p2, val);
                }
            }

            OP_BIT_AND | OP_BIT_OR | OP_SHIFT_LEFT | OP_SHIFT_RIGHT | OP_ADD | OP_SUBTRACT
            | OP_MULTIPLY | OP_DIVIDE | OP_REMAINDER | OP_CONCAT => {
                // r[p3] = r[p1] + r[p2]
                match (r.get(&p1).copied(), r.get(&p2).copied()) {
                    (Some(a), Some(b)) => {
                        r.insert(p3, if matches!(a, DataType::Null) { b } else { a });
                    }

                    (Some(v), None) => {
                        r.insert(p3, v);
                    }

                    (None, Some(v)) => {
                        r.insert(p3, v);
                    }

                    _ => {}
                }

                match (n.get(&p1).copied(), n.get(&p2).copied()) {
                    (Some(a), Some(b)) => {
                        n.insert(p3, a || b);
                    }

                    (None, Some(b)) => {
                        n.insert(p3, b);
                    }

                    (Some(a), None) => {
                        n.insert(p3, a);
                    }

                    _ => {}
                }
            }

            OP_RESULT_ROW => {
                // output = r[p1 .. p1 + p2]
                let mut output = Vec::with_capacity(p2 as usize);
                let mut nullable = Vec::with_capacity(p2 as usize);

                for i in p1..p1 + p2 {
                    output.push(SqliteTypeInfo(r.remove(&i).unwrap_or(DataType::Null)));

                    nullable.push(if n.remove(&i).unwrap_or(true) {
                        None
                    } else {
                        Some(false)
                    });
                }

                return Ok((output, nullable));
            }

            _ => {
                // ignore unsupported operations
                // if we fail to find an r later, we just give up
            }
        }

        program_i += 1;
    }

    // no rows
    Ok((vec![], vec![]))
}
