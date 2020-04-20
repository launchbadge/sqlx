use std::ops::{Deref, DerefMut};

use bytes::{Buf, Bytes};
use log::Level;

use crate::error::Error;
use crate::io::{BufStream, Decode};
use crate::net::{MaybeTlsStream, Socket};
use crate::postgres::message::{Message, MessageFormat, Response};
use crate::postgres::{PgConnectOptions, PgDatabaseError, PgSeverity};

// the stream is a separate type from the connection to uphold the invariant where an instantiated
// [PgConnection] is a **valid** connection to postgres

// when a new connection is asked for, we work directly on the [PgStream] type until the
// connection is fully established

// in other words, `self` in any PgConnection method is a live connection to postgres that
// is fully prepared to receive queries

pub struct PgStream(BufStream<MaybeTlsStream<Socket>>);

impl PgStream {
    #[inline]
    pub(super) async fn connect(options: &PgConnectOptions) -> Result<Self, Error> {
        Ok(PgStream(BufStream::new(MaybeTlsStream::Raw(
            Socket::connect(&options.host, options.port).await?,
        ))))
    }

    // Expect a specific type and format
    pub(super) async fn recv_expect<T: Decode>(
        &mut self,
        format: MessageFormat,
    ) -> Result<T, Error> {
        let message = self.recv().await?;

        if message.format != format {
            return Err(err_protocol!(
                "expecting {:?} but received {:?}",
                format,
                message.format
            ));
        }

        message.decode()
    }

    // Get the next message from the server
    // May wait for more data from the server
    pub(super) async fn recv(&mut self) -> Result<Message, Error> {
        loop {
            // all packets in postgres start with a 5-byte header
            // this header contains the message type and the total length of the message
            let mut header: Bytes = self.0.read(5).await?;

            let format = MessageFormat::try_from_u8(header.get_u8())?;
            let size = (header.get_u32() - 4) as usize;

            let contents = self.0.read(size).await?;

            match format {
                MessageFormat::ErrorResponse => {
                    // An error returned from the database server.
                    return Err(PgDatabaseError(Response::decode(contents)?).into());
                }

                MessageFormat::NoticeResponse => {
                    // do we need this to be _more_ configurable?
                    // if you are reading this comment and think so, open an issue

                    let notice = Response::decode(contents)?;

                    let lvl = match notice.severity() {
                        PgSeverity::Fatal | PgSeverity::Panic | PgSeverity::Error => Level::Error,
                        PgSeverity::Warning => Level::Warn,
                        PgSeverity::Notice => Level::Info,
                        PgSeverity::Debug => Level::Debug,
                        PgSeverity::Info => Level::Trace,
                        PgSeverity::Log => Level::Trace,
                    };

                    // HACK: massive hack here so we can force a log at a specific module path
                    // FIXME: if you open a PR to make this hack go away but keep the functionality
                    //        it will straight up make my day

                    if lvl <= log::STATIC_MAX_LEVEL && lvl <= log::max_level() {
                        log::__private_api_log(
                            format_args!("{}", notice.message()),
                            lvl,
                            &(
                                "sqlx::postgres::notice",
                                "sqlx::postgres::notice",
                                file!(),
                                line!(),
                            ),
                        );
                    }

                    continue;
                }

                _ => {}
            }

            return Ok(Message { format, contents });
        }
    }
}

impl Deref for PgStream {
    type Target = BufStream<MaybeTlsStream<Socket>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PgStream {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
