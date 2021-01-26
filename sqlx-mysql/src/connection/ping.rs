use sqlx_core::{Result, Runtime};

use crate::protocol::{OkPacket, Ping};

// PING is very simple
// send the COM_PING packet
// should receive an OK

macro_rules! impl_ping {
    ($(@$blocking:ident)? $self:ident) => {{
        $self.stream.write_packet(&Ping)?;

        // STATE: remember that we are expecting an OK packet
        $self.begin_simple_command();

        let _ok: OkPacket = read_packet!($(@$blocking)? $self.stream)
            .deserialize_with($self.capabilities)?;

        // STATE: received OK packet
        $self.end_command();

        Ok(())
    }};
}

impl<Rt: Runtime> super::MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn ping_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        impl_ping!(self)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn ping_blocking(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_ping!(@blocking self)
    }
}
