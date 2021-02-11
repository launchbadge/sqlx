use sqlx_core::{Result, Runtime};

use crate::protocol::{Ping, ResultPacket};

// PING is very simple
// send the COM_PING packet
// should receive an OK

macro_rules! impl_ping {
    ($(@$blocking:ident)? $self:ident) => {{
        $self.stream.write_packet(&Ping)?;

        // STATE: remember that we are expecting an OK packet
        $self.commands.begin();

        let _ok = read_packet!($(@$blocking)? $self.stream)
            .deserialize_with::<ResultPacket, _>($self.capabilities)?
            .into_result()?;

        // STATE: received OK packet
        $self.commands.end();

        Ok(())
    }};
}

impl<Rt: Runtime> super::MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn ping_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        flush!(self);
        impl_ping!(self)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn ping_blocking(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        flush!(@blocking self);
        impl_ping!(@blocking self)
    }
}
