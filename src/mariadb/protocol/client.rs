// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

// TODO: Handle lengths which are greater than 3 bytes
// Either break the backet into several smaller ones, or
// return error
// TODO: Handle different Capabilities for server and client
// TODO: Handle when capability is set, but field is None

use super::packets::{SetOptionOptions, ShutdownOptions};

// This is an enum of text protocol packet tags.
// Tags are the 5th byte of the packet (1st byte of packet body)
// and are used to determine which type of query was sent.
// The name of the enum variant represents the type of query, and
// the value is the byte value required by the server.
pub enum TextProtocol {
    ComChangeUser = 0x11,
    ComDebug = 0x0D,
    ComInitDb = 0x02,
    ComPing = 0x0e,
    ComProcessKill = 0xC,
    ComQuery = 0x03,
    ComQuit = 0x01,
    ComResetConnection = 0x1F,
    ComSetOption = 0x1B,
    ComShutdown = 0x0A,
    ComSleep = 0x00,
    ComStatistics = 0x09,
}

// Helper method to easily transform into u8
impl Into<u8> for TextProtocol {
    fn into(self) -> u8 {
        self as u8
    }
}
