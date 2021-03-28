use sqlx_core::{Error, Result, Runtime};

use crate::protocol::backend::{BackendMessage, BackendMessageType};
use crate::PgConnection;

impl<Rt: Runtime> PgConnection<Rt> {
    fn handle_message_in_flush(&mut self, message: BackendMessage) -> Result<bool> {
        match message.ty {
            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                return Ok(true);
            }

            _ => {}
        }

        Ok(false)
    }
}

macro_rules! impl_flush {
    ($(@$blocking:ident)? $self:ident) => {{
        while $self.pending_ready_for_query_count > 0 {
            loop {
                let message = read_message!($(@$blocking)? $self.stream);

                match message {
                    Ok(message) => {
                        if $self.handle_message_in_flush(message)? {
                            break;
                        }
                    }

                    Err(error) => {
                        if matches!(error, Error::Database(_)) {
                            // log database errors instead of failing on them
                            // during a flush
                            log::error!("{}", error);
                        } else {
                            return Err(error);
                        }
                    }
                }

            }
        }

        Ok(())
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn flush_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        impl_flush!(self)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn flush_blocking(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_flush!(@blocking self)
    }
}

macro_rules! flush {
    (@blocking $self:ident) => {
        $self.flush_blocking()?
    };

    ($self:ident) => {
        $self.flush_async().await?
    };
}
