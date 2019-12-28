use crate::error::DatabaseError;
use crate::mysql::protocol::ErrPacket;

pub struct MySqlError(pub(super) ErrPacket);

impl DatabaseError for MySqlError {
    fn message(&self) -> &str {
        &*self.0.error_message
    }
}

impl_fmt_error!(MySqlError);
