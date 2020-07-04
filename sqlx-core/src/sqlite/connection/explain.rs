use crate::error::Error;
use crate::query_as::query_as;
use crate::sqlite::type_info::DataType;
use crate::sqlite::{SqliteConnection, SqliteTypeInfo};
use hashbrown::HashMap;

const OP_INIT: &str = "Init";
const OP_GOTO: &str = "Goto";
const OP_COLUMN: &str = "Column";
const OP_AGG_STEP: &str = "AggStep";
const OP_MOVE: &str = "Move";
const OP_COPY: &str = "Copy";
const OP_SCOPY: &str = "SCopy";
const OP_INT_COPY: &str = "IntCopy";
const OP_STRING8: &str = "String8";
const OP_INT64: &str = "Int64";
const OP_INTEGER: &str = "Integer";
const OP_REAL: &str = "Real";
const OP_NOT: &str = "Not";
const OP_BLOB: &str = "Blob";
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

fn to_type(op: &str) -> DataType {
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
) -> Result<Vec<SqliteTypeInfo>, Error> {
    let mut r = HashMap::<i64, DataType>::with_capacity(6);

    let program =
        query_as::<_, (i64, String, i64, i64, i64, String)>(&*format!("EXPLAIN {}", query))
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
            }

            OP_AGG_STEP => {
                if p4.starts_with("count(") {
                    // count(_) -> INTEGER
                    r.insert(p3, DataType::Int64);
                } else if let Some(v) = r.get(&p2).copied() {
                    // r[p3] = AGG ( r[p2] )
                    r.insert(p3, v);
                }
            }

            OP_COPY | OP_MOVE | OP_SCOPY | OP_INT_COPY => {
                // r[p2] = r[p1]
                if let Some(v) = r.get(&p1).copied() {
                    r.insert(p2, v);
                }
            }

            OP_OR | OP_AND | OP_BLOB | OP_COUNT | OP_REAL | OP_STRING8 | OP_INTEGER | OP_ROWID => {
                // r[p2] = <value of constant>
                r.insert(p2, to_type(&opcode));
            }

            OP_NOT => {
                // r[p2] = NOT r[p1]
                if let Some(a) = r.get(&p1).copied() {
                    r.insert(p2, a);
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
            }

            OP_RESULT_ROW => {
                // output = r[p1 .. p1 + p2]
                let mut output = Vec::with_capacity(p2 as usize);
                for i in p1..p1 + p2 {
                    output.push(SqliteTypeInfo(r.remove(&i).unwrap_or(DataType::Null)));
                }

                return Ok(output);
            }

            _ => {
                // ignore unsupported operations
                // if we fail to find an r later, we just give up
            }
        }

        program_i += 1;
    }

    // no rows
    Ok(vec![])
}
