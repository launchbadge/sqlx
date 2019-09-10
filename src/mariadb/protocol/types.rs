#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StmtExecFlag(pub u8);
impl StmtExecFlag {
    pub const CURSOR_FOR_UPDATE: StmtExecFlag = StmtExecFlag(2);
    pub const NO_CURSOR: StmtExecFlag = StmtExecFlag(0);
    pub const READ_ONLY: StmtExecFlag = StmtExecFlag(1);
    pub const SCROLLABLE_CURSOR: StmtExecFlag = StmtExecFlag(3);
}
