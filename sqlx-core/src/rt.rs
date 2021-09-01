//! This module contains feature specific to only certain runtimes

#[cfg(any(
    feature = "runtime-actix-native-tls",
    feature = "runtime-tokio-native-tls",
    feature = "runtime-actix-rustls",
    feature = "runtime-tokio-rustls",
    feature = "rt-docs",
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "runtime-actix-native-tls",
        feature = "runtime-tokio-native-tls",
        feature = "runtime-actix-rustls",
        feature = "runtime-tokio-rustls",
    )))
)]
pub use sqlx_rt::set_runtime;
