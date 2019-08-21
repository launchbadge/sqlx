pub enum ProtocolType {
    Text,
    Binary,
}

bitflags! {
    pub struct Capabilities: u128 {
        const CLIENT_MYSQL = 1;
        const FOUND_ROWS = 1 << 1;
        const CONNECT_WITH_DB = 1 << 3;
        const COMPRESS = 1 << 5;
        const LOCAL_FILES = 1 << 7;
        const IGNORE_SPACE = 1 << 8;
        const CLIENT_PROTOCOL_41 = 1 << 9;
        const CLIENT_INTERACTIVE = 1 << 10;
        const SSL = 1 << 11;
        const TRANSACTIONS = 1 << 12;
        const SECURE_CONNECTION = 1 << 13;
        const MULTI_STATEMENTS = 1 << 16;
        const MULTI_RESULTS = 1 << 17;
        const PS_MULTI_RESULTS = 1 << 18;
        const PLUGIN_AUTH = 1 << 19;
        const CONNECT_ATTRS = 1 << 20;
        const PLUGIN_AUTH_LENENC_CLIENT_DATA = 1 << 21;
        const CLIENT_SESSION_TRACK = 1 << 23;
        const CLIENT_DEPRECATE_EOF = 1 << 24;
        const MARIA_DB_CLIENT_PROGRESS = 1 << 32;
        const MARIA_DB_CLIENT_COM_MULTI = 1 << 33;
        const MARIA_CLIENT_STMT_BULK_OPERATIONS = 1 << 34;
    }
}

bitflags! {
    pub struct FieldDetailFlag: u16 {
        const NOT_NULL = 1;
        const PRIMARY_KEY = 2;
        const UNIQUE_KEY = 4;
        const MULTIPLE_KEY = 8;
        const BLOB = 16;
        const UNSIGNED = 32;
        const ZEROFILL_FLAG = 64;
        const BINARY_COLLATION = 128;
        const ENUM = 256;
        const AUTO_INCREMENT = 512;
        const TIMESTAMP = 1024;
        const SET = 2048;
        const NO_DEFAULT_VALUE_FLAG = 4096;
        const ON_UPDATE_NOW_FLAG = 8192;
        const NUM_FLAG = 32768;
    }
}

bitflags! {
    pub struct ServerStatusFlag: u16 {
        const SERVER_STATUS_IN_TRANS = 1;
        const SERVER_STATUS_AUTOCOMMIT = 2;
        const SERVER_MORE_RESULTS_EXISTS = 8;
        const SERVER_QUERY_NO_GOOD_INDEX_USED = 16;
        const SERVER_QUERY_NO_INDEX_USED = 32;
        const SERVER_STATUS_CURSOR_EXISTS = 64;
        const SERVER_STATUS_LAST_ROW_SENT = 128;
        const SERVER_STATUS_DB_DROPPED = 1 << 8;
        const SERVER_STATUS_NO_BACKSLASH_ESAPES = 1 << 9;
        const SERVER_STATUS_METADATA_CHANGED = 1 << 10;
        const SERVER_QUERY_WAS_SLOW = 1 << 11;
        const SERVER_PS_OUT_PARAMS = 1 << 12;
        const SERVER_STATUS_IN_TRANS_READONLY = 1 << 13;
        const SERVER_SESSION_STATE_CHANGED = 1 << 14;
    }
}

pub enum SessionChangeType {
    SessionTrackSystemVariables = 0,
    SessionTrackSchema = 1,
    SessionTrackStateChange = 2,
    SessionTrackGTIDS = 3,
    SessionTrackTransactionCharacteristics = 4,
    SessionTrackTransactionState = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldType(pub u8);
impl FieldType {
    pub const MYSQL_TYPE_DECIMAL: FieldType = FieldType(0);
    pub const MYSQL_TYPE_TINY: FieldType = FieldType(1);
    pub const MYSQL_TYPE_SHORT: FieldType = FieldType(2);
    pub const MYSQL_TYPE_LONG: FieldType = FieldType(3);
    pub const MYSQL_TYPE_FLOAT: FieldType = FieldType(4);
    pub const MYSQL_TYPE_DOUBLE: FieldType = FieldType(5);
    pub const MYSQL_TYPE_NULL: FieldType = FieldType(6);
    pub const MYSQL_TYPE_TIMESTAMP: FieldType = FieldType(7);
    pub const MYSQL_TYPE_LONGLONG: FieldType = FieldType(8);
    pub const MYSQL_TYPE_INT24: FieldType = FieldType(9);
    pub const MYSQL_TYPE_DATE: FieldType = FieldType(10);
    pub const MYSQL_TYPE_TIME: FieldType = FieldType(11);
    pub const MYSQL_TYPE_DATETIME: FieldType = FieldType(12);
    pub const MYSQL_TYPE_YEAR: FieldType = FieldType(13);
    pub const MYSQL_TYPE_NEWDATE: FieldType = FieldType(14);
    pub const MYSQL_TYPE_VARCHAR: FieldType = FieldType(15);
    pub const MYSQL_TYPE_BIT: FieldType = FieldType(16);
    pub const MYSQL_TYPE_TIMESTAMP2: FieldType = FieldType(17);
    pub const MYSQL_TYPE_DATETIME2: FieldType = FieldType(18);
    pub const MYSQL_TYPE_TIME2: FieldType = FieldType(19);
    pub const MYSQL_TYPE_JSON: FieldType = FieldType(245);
    pub const MYSQL_TYPE_NEWDECIMAL: FieldType = FieldType(246);
    pub const MYSQL_TYPE_ENUM: FieldType = FieldType(247);
    pub const MYSQL_TYPE_SET: FieldType = FieldType(248);
    pub const MYSQL_TYPE_TINY_BLOB: FieldType = FieldType(249);
    pub const MYSQL_TYPE_MEDIUM_BLOB: FieldType = FieldType(250);
    pub const MYSQL_TYPE_LONG_BLOB: FieldType = FieldType(251);
    pub const MYSQL_TYPE_BLOB: FieldType = FieldType(252);
    pub const MYSQL_TYPE_VAR_STRING: FieldType = FieldType(253);
    pub const MYSQL_TYPE_STRING: FieldType = FieldType(254);
    pub const MYSQL_TYPE_GEOMETRY: FieldType = FieldType(255);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StmtExecFlag(pub u8);
impl StmtExecFlag {
    pub const NO_CURSOR: StmtExecFlag = StmtExecFlag(0);
    pub const READ_ONLY: StmtExecFlag = StmtExecFlag(1);
    pub const CURSOR_FOR_UPDATE: StmtExecFlag = StmtExecFlag(2);
    pub const SCROLLABLE_CURSOR: StmtExecFlag = StmtExecFlag(3);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParamFlag(pub u8);
impl ParamFlag {
    pub const NONE: ParamFlag = ParamFlag(0);
    pub const UNSIGNED: ParamFlag = ParamFlag(128);
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities::CLIENT_PROTOCOL_41
    }
}

impl Default for ServerStatusFlag {
    fn default() -> Self {
        ServerStatusFlag::SERVER_STATUS_IN_TRANS
    }
}

impl Default for FieldDetailFlag {
    fn default() -> Self {
        FieldDetailFlag::NOT_NULL
    }
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::MYSQL_TYPE_DECIMAL
    }
}

impl Default for StmtExecFlag {
    fn default() -> Self {
        StmtExecFlag::NO_CURSOR
    }
}

impl Default for ParamFlag {
    fn default() -> Self {
        ParamFlag::UNSIGNED
    }
}

#[cfg(test)]
mod test {
    use super::super::{decode::Decoder, types::Capabilities};
    use bytes::Bytes;

    #[test]
    fn it_decodes_capabilities() {
        let buf = Bytes::from(b"\xfe\xf7".to_vec());
        let mut decoder = Decoder::new(buf);
        Capabilities::from_bits_truncate(decoder.decode_int_u16().into());
    }
}
