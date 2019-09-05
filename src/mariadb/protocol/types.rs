
bitflags::bitflags! {
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

bitflags::bitflags! {
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
pub struct StmtExecFlag(pub u8);
impl StmtExecFlag {
    pub const CURSOR_FOR_UPDATE: StmtExecFlag = StmtExecFlag(2);
    pub const NO_CURSOR: StmtExecFlag = StmtExecFlag(0);
    pub const READ_ONLY: StmtExecFlag = StmtExecFlag(1);
    pub const SCROLLABLE_CURSOR: StmtExecFlag = StmtExecFlag(3);
}
