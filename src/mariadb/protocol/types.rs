
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
