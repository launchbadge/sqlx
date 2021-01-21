// pick a default runtime
// this is so existing applications in SQLx pre 0.6 work and to
// make it more convenient, if your application only uses 1 runtime (99%+)
// most of the time you won't have to worry about picking the runtime
mod default {
    #[cfg(all(not(all(feature = "async-std", feature = "tokio")), feature = "actix"))]
    pub use sqlx_core::Actix as Runtime;
    #[cfg(feature = "async-std")]
    pub use sqlx_core::AsyncStd as Runtime;
    #[cfg(all(not(feature = "async-std"), feature = "tokio"))]
    pub use sqlx_core::Tokio as Runtime;

    #[cfg(all(
        not(any(feature = "async-std", feature = "tokio", feature = "actix")),
        feature = "blocking"
    ))]
    pub use crate::Blocking as Runtime;

    // when there is no async runtime, and the blocking runtime is not present
    // the unit type is implemented for Runtime, this is only to allow the
    // lib to compile, the lib is mostly useless in this state
    #[cfg(not(any(
        feature = "async-std",
        feature = "actix",
        feature = "tokio",
        feature = "blocking"
    )))]
    pub type Runtime = ();
}

/// The default runtime in use by SQLx when one is unspecified.
///
/// Following the crate features for each runtime are activated, a default is picked
/// by following a priority list. The actual sorting here is mostly arbitrary (what is
/// important is that there _is_ a stable ordering).
///
/// 1.   [`AsyncStd`][crate::AsyncStd]
/// 2.   [`Tokio`][crate::Tokio]
/// 3.   [`Actix`][crate::Actix]
/// 4.   [`Blocking`][crate::Blocking]
/// 5.   `()` - No runtime selected (nothing is possible)
///
/// The intent is to allow the following to cleanly work, regardless of the enabled runtime,
/// if only one runtime is enabled.
///
/// <br>
///
/// ```rust,ignore
/// use sqlx::postgres::{PgConnection, PgConnectOptions};
///
/// // PgConnection<Rt = sqlx::DefaultRuntime>
/// let conn: PgConnection = PgConnectOptions::new()
///     .host("localhost")
///     .username("postgres")
///     .password("password")
///     // .connect()?; // for Blocking runtime
///     .connect().await?; // for Async runtimes
/// ```
///
#[allow(clippy::module_name_repetitions)]
pub type DefaultRuntime = default::Runtime;
