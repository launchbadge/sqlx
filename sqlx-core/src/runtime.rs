use crate::io::Stream;

// #[cfg(feature = "_mock")]
// #[doc(hidden)]
// pub mod mock;

#[cfg(feature = "async-std")]
#[path = "runtime/async_std.rs"]
mod async_std_;

#[cfg(feature = "actix")]
#[path = "runtime/actix.rs"]
mod actix_;

#[cfg(feature = "tokio")]
#[path = "runtime/tokio.rs"]
mod tokio_;

#[cfg(feature = "actix")]
pub use actix_::Actix;
#[cfg(feature = "async-std")]
pub use async_std_::AsyncStd;
// #[cfg(feature = "_mock")]
// pub use mock::Mock;
#[cfg(feature = "tokio")]
pub use tokio_::Tokio;

/// Describes a set of types and functions used to open and manage IO resources within SQLx.
///
/// In the greater ecosystem we have several choices for an asynchronous runtime (executor) to
/// schedule and interact with our futures. Libraries that wish to be generally available have
/// tended to either pick one (and allow compatibility wrappers to others) or use mutually-exclusive
/// cargo feature flags to pick between runtimes. Each of these approaches have their own
/// problems.
///
/// In SQLx, most types and traits are parameterized with a `Rt: Runtime` bound. Asynchronous
/// implementations of `Runtime` are available for [**async-std**](https://async.rs/),
/// [**Tokio**](https://tokio.rs/), and [**Actix**](https://actix.rs/) (given
/// those crate features are activated).
///
/// -   [`AsyncStd`]
/// -   [`Tokio`]
/// -   [`Actix`]
///
/// Additionally, a `std` blocking runtime is provided. This is intended for use in
/// environments where asynchronous IO either doesn't make sense or isn't available.
///
/// -   [`Blocking`][crate::blocking::Blocking]
///
pub trait Runtime: 'static + Send + Sync + Sized {
    #[doc(hidden)]
    type TcpStream: 'static + Send + Sync;
}

/// Marks a [`Runtime`] as being capable of handling asynchronous execution.
// Provided so that attempting to use the asynchronous methods with the
// Blocking runtime will error at compile-time as opposed to runtime.
pub trait Async: Runtime
where
    // NOTE: This requires a **pervasive** bound for everything that needs
    //       to bound on <Runtime>. Remove if you can think of something else that
    //       allows us to both allow polymorphic read/write on streams across
    //       runtimes _and_ not depend on <BoxFuture>. When GATs are stabilized,
    //       we should be able to switch to that and remove this HRTB.
    Self::TcpStream: for<'s> Stream<'s, Self>,
{
    #[cfg(feature = "async")]
    #[doc(hidden)]
    fn connect_tcp_async(
        host: &str,
        port: u16,
    ) -> futures_util::future::BoxFuture<'_, std::io::Result<Self::TcpStream>>;
}

// when no runtime is available
// we implement `()` for it to allow the lib to still compile
#[cfg(not(any(
    feature = "async-std",
    feature = "actix",
    feature = "tokio",
    feature = "blocking"
)))]
impl Runtime for () {
    type TcpStream = ();
}

// pick a default runtime
// this is so existing applications in SQLx pre 0.6 work and to
// make it more convenient, if your application only uses 1 runtime (99%+)
// most of the time you won't have to worry about picking the runtime
mod default {
    #[cfg(feature = "async-std")]
    pub type Runtime = super::AsyncStd;

    #[cfg(all(not(feature = "async-std"), feature = "tokio"))]
    pub type Runtime = super::Tokio;

    #[cfg(all(not(all(feature = "async-std", feature = "tokio")), feature = "actix"))]
    pub type Runtime = super::Actix;

    #[cfg(all(
        not(any(feature = "async-std", feature = "tokio", feature = "actix")),
        feature = "blocking"
    ))]
    pub type Runtime = crate::Blocking;

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
/// 1.   [`AsyncStd`]
/// 2.   [`Tokio`]
/// 3.   [`Actix`]
/// 4.   [`Blocking`]
/// 5.   `()` – No runtime selected (nothing is possible)
///
/// The intent is to allow the following to cleanly work, regardless of the enabled runtime,
/// if only one runtime is enabled.
///
/// <br>
///
/// ```rust,ignore
/// use sqlx::postgres::{PgConnection, PgConnectOptions};
/// use sqlx::prelude::*;
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
