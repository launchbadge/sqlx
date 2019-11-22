mod com_debug;
mod com_init_db;
mod com_ping;
mod com_process_kill;
mod com_query;
mod com_quit;
mod com_reset_conn;
mod com_set_option;
mod com_sleep;
mod com_statistics;

pub use com_debug::ComDebug;
pub use com_init_db::ComInitDb;
pub use com_ping::ComPing;
pub use com_process_kill::ComProcessKill;
pub use com_query::ComQuery;
pub use com_quit::ComQuit;
pub use com_reset_conn::ComResetConnection;
pub use com_set_option::{ComSetOption, SetOptionOptions};
pub use com_sleep::ComSleep;
pub use com_statistics::ComStatistics;

// This is an enum of text protocol packet tags.
// Tags are the 5th byte of the packet (1st byte of packet body)
// and are used to determine which type of query was sent.
// The name of the enum variant represents the type of query, and
// the value is the byte value required by the server.
enum TextProtocol {
    ComChangeUser = 0x11,
    ComDebug = 0x0D,
    ComInitDb = 0x02,
    ComPing = 0x0e,
    ComProcessKill = 0x0C,
    ComQuery = 0x03,
    ComQuit = 0x01,
    ComResetConnection = 0x1F,
    ComSetOption = 0x1B,
    ComShutdown = 0x0A,
    ComSleep = 0x00,
    ComStatistics = 0x09,
}
