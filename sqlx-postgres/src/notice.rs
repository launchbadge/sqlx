use std::pin::Pin;
use futures_util::{Sink, SinkExt};
use sqlx_core::error::{BoxDynError, Error};
use sqlx_core::logger::log_level_to_tracing_level;
use crate::message::PgNotice;

/// Sink for Postgres `NoticeResponse`s.
pub struct PgNoticeSink {
    inner: SinkInner
}

enum SinkInner {
    Discard,
    Log,
    Closure(Box<dyn FnMut(PgNotice) -> Result<(), BoxDynError> + Send + Sync + 'static>),
    Wrapped(Pin<Box<dyn Sink<PgNotice, Error = BoxDynError> + Send + Sync + 'static>>),
}

impl PgNoticeSink {
    /// Discard all `NoticeResponse`s.
    pub fn discard() -> Self {
        PgNoticeSink {
            inner: SinkInner::Discard,
        }
    }

    /// Log `NoticeResponse`s according to severity level under the target `sqlx::postgres::notice`.
    ///
    /// | Postgres Severity Level   | `log`/`tracing` Level |
    /// | ------------------------- | --------------------- |
    /// | `PANIC`, `FATAL`, `ERROR` | `ERROR`               |
    /// | `WARNING`                 | `WARN`                |
    /// | `NOTICE`                  | `INFO`                |
    /// | `DEBUG`                   | `DEBUG`               |
    /// | `INFO`, `LOG`             | `TRACE`               |
    ///
    /// This is the default behavior of new `PgConnection`s.
    ///
    /// To instead consume `NoticeResponse`s directly as [`PgNotice`]s, see:
    ///
    /// * [`PgNoticeSink::closure()`]
    /// * [`PgNoticeSink::wrap()`]
    /// * [`PgConnection::set_notice_sink()`][crate::PgConnection::set_notice_sink()]
    pub fn log() -> Self {
        PgNoticeSink {
            inner: SinkInner::Log
        }
    }

    /// Supply a closure to handle [`PgNotice`]s.
    ///
    /// Errors will be bubbled up by the connection as [`Error::Internal`].
    ///
    /// # Warning: Do Not Block!
    ///
    /// The closure is invoked directly by the connection, so it should not block if it is unable
    /// to immediately consume the message.
    ///
    /// Instead, use [`Self::wrap()`] to provide a [`futures::Sink`] implementation.
    pub fn closure(f: impl FnMut(PgNotice) -> Result<(), BoxDynError> + Send + Sync + 'static) -> Self {
        PgNoticeSink {
            inner: SinkInner::Closure(Box::new(f)),
        }
    }

    /// Supply a [`futures::Sink`] to handle [`PgNotice`]s.
    ///
    /// Errors will be bubbled up by the connection as [`Error::Internal`].
    pub fn wrap(sink: impl Sink<PgNotice, Error = BoxDynError> + Send + Sync + 'static) -> Self {
        PgNoticeSink {
            inner: SinkInner::Wrapped(Box::pin(sink)),
        }
    }

    pub(crate) async fn consume(&mut self, notice: PgNotice) -> Result<(), Error> {
        match &mut self.inner {
            SinkInner::Discard => Ok(()),
            SinkInner::Log => {
                log_notice(notice);
                Ok(())
            }
            SinkInner::Closure(f) => f(notice).map_err(Error::Internal),
            SinkInner::Wrapped(sink) => {
                sink.as_mut().send(notice).await.map_err(Error::Internal)
            }
        }
    }
}

fn log_notice(notice: PgNotice) {
    let tracing_level = notice.severity().to_tracing_level();

    let log_is_enabled = log::log_enabled!(
            target: "sqlx::postgres::notice",
            notice.severity().to_log_level()
        ) || sqlx_core::private_tracing_dynamic_enabled!(
            target: "sqlx::postgres::notice",
            tracing_level
        );

    if log_is_enabled {
        sqlx_core::private_tracing_dynamic_event!(
            target: "sqlx::postgres::notice",
            tracing_level,
            severity=%notice.severity(),
            code=%notice.code(),
            "{}",
            notice.message()
        );
    }
}
