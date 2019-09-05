// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

mod capabilities;
mod connect;
mod encode;
mod error_code;
mod field;
mod server_status;
mod response;

pub use capabilities::Capabilities;
pub use connect::{
    AuthenticationSwitchRequest, HandshakeResponsePacket, InitialHandshakePacket, SslRequest,
};
pub use response::{
    OkPacket, EofPacket, ErrPacket, ResultRow,
};
pub use encode::Encode;
pub use error_code::ErrorCode;
pub use field::{FieldType, ParameterFlag};
pub use server_status::ServerStatusFlag;
