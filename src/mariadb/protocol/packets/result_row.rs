use crate::mariadb::{ResultRowText, ResultRowBinary};

#[derive(Debug)]
pub struct ResultRow {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Vec<Option<bytes::Bytes>>
}

impl From<ResultRowText> for ResultRow {
    fn from(row: ResultRowText) -> Self {
        ResultRow {
            length: row.length,
            seq_no: row.seq_no,
            columns: row.columns,
        }
    }
}


impl From<ResultRowBinary> for ResultRow {
    fn from(row: ResultRowBinary) -> Self {
        ResultRow {
            length: row.length,
            seq_no: row.seq_no,
            columns: row.columns,
        }
    }
}
