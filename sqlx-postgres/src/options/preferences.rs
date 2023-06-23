use std::sync::Arc;

use crate::PgSeverity;

#[derive(Default, Clone)]
pub(crate) struct PgClientPreferences {
    pub(crate) notice_response_log_levels_fn: Option<Arc<NoticeResponseLogLevels>>,
}

impl std::fmt::Debug for PgClientPreferences {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgClientPreferences")
            .finish_non_exhaustive()
    }
}

impl PgClientPreferences {
    pub(crate) fn notice_response_log_levels(
        &self,
        severity: PgSeverity,
    ) -> (log::Level, tracing::Level) {
        let function = self
            .notice_response_log_levels_fn
            .as_deref()
            .unwrap_or(&default_notice_response_log_levels);
        function(severity)
    }
}

/// Determines the level at which NoticeResponses are logged with [`log`] and
/// [`tracing`].
pub(crate) type NoticeResponseLogLevels =
    dyn Fn(PgSeverity) -> (log::Level, tracing::Level) + Send + Sync;

fn default_notice_response_log_levels(severity: PgSeverity) -> (log::Level, tracing::Level) {
    use log::Level;
    match severity {
        PgSeverity::Fatal | PgSeverity::Panic | PgSeverity::Error => {
            (Level::Error, tracing::Level::ERROR)
        }
        PgSeverity::Warning => (Level::Warn, tracing::Level::WARN),
        PgSeverity::Notice => (Level::Info, tracing::Level::INFO),
        PgSeverity::Debug => (Level::Debug, tracing::Level::DEBUG),
        PgSeverity::Info | PgSeverity::Log => (Level::Trace, tracing::Level::TRACE),
    }
}

/// Compute the tracing level from a log level.
///
/// (The other direction is not possible; tracing levels are not defined
/// exhaustively.)
pub(crate) fn compute_tracing_level(level: log::Level) -> tracing::Level {
    match level {
        log::Level::Error => tracing::Level::ERROR,
        log::Level::Warn => tracing::Level::WARN,
        log::Level::Info => tracing::Level::INFO,
        log::Level::Debug => tracing::Level::DEBUG,
        log::Level::Trace => tracing::Level::TRACE,
    }
}
