pub mod com_stmt_prepare;
pub mod com_stmt_prepare_ok;
pub mod com_stmt_prepare_resp;
pub mod com_stmt_close;
pub mod com_stmt_exec;
pub mod com_stmt_fetch;
pub mod com_stmt_reset;
pub mod result_row;

pub use com_stmt_prepare::ComStmtPrepare;
pub use com_stmt_prepare_ok::ComStmtPrepareOk;
pub use com_stmt_prepare_resp::ComStmtPrepareResp;
pub use com_stmt_close::ComStmtClose;
pub use com_stmt_exec::ComStmtExec;
pub use com_stmt_fetch::ComStmtFetch;
pub use com_stmt_reset::ComStmtReset;

pub enum BinaryProtocol {
    ComStmtPrepare = 0x16,
    ComStmtExec = 0x17,
    ComStmtClose = 0x19,
    ComStmtReset = 0x1A,
    ComStmtFetch = 0x1C,
}

// Helper method to easily transform into u8
impl Into<u8> for BinaryProtocol {
    fn into(self) -> u8 {
        self as u8
    }
}

