use std::convert::TryFrom;

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

#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u8"]
pub enum FieldType {
    MysqlTypeDecimal = 0,
    MysqlTypeTiny = 1,
    MysqlTypeShort = 2,
    MysqlTypeLong = 3,
    MysqlTypeFloat = 4,
    MysqlTypeDouble = 5,
    MysqlTypeNull = 6,
    MysqlTypeTimestamp = 7,
    MysqlTypeLonglong = 8,
    MysqlTypeInt24 = 9,
    MysqlTypeDate = 10,
    MysqlTypeTime = 11,
    MysqlTypeDatetime = 12,
    MysqlTypeYear = 13,
    MysqlTypeNewdate = 14,
    MysqlTypeVarchar = 15,
    MysqlTypeBit = 16,
    MysqlTypeTimestamp2 = 17,
    MysqlTypeDatetime2 = 18,
    MysqlTypeTime2 = 19,
    MysqlTypeJson = 245,
    MysqlTypeNewdecimal = 246,
    MysqlTypeEnum = 247,
    MysqlTypeSet = 248,
    MysqlTypeTinyBlob = 249,
    MysqlTypeMediumBlob = 250,
    MysqlTypeLongBlob = 251,
    MysqlTypeBlob = 252,
    MysqlTypeVarString = 253,
    MysqlTypeString = 254,
    MysqlTypeGeometry = 255,
}

#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u8"]
pub enum StmtExecFlag {
    NoCursor = 0,
    ReadOnly = 1,
    CursorForUpdate = 2,
    ScrollableCursor = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType = "u8"]
pub enum ParamFlag {
    Unsigned = 128,
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
        FieldType::MysqlTypeDecimal
    }
}

impl Default for StmtExecFlag {
    fn default() -> Self {
        StmtExecFlag::NoCursor
    }
}

impl Default for ParamFlag {
    fn default() -> Self {
        ParamFlag::Unsigned
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
