//! Core runtime support for SQLx. **Semver-exempt**, not for general use.

#[cfg(not(any(
    feature = "runtime-actix-native-tls",
    feature = "runtime-async-std-native-tls",
    feature = "runtime-tokio-native-tls",
    feature = "runtime-actix-rustls",
    feature = "runtime-async-std-rustls",
    feature = "runtime-tokio-rustls",
)))]
compile_error!(
    "one of the features ['runtime-actix-native-tls', 'runtime-async-std-native-tls', \
     'runtime-tokio-native-tls', 'runtime-actix-rustls', 'runtime-async-std-rustls', \
     'runtime-tokio-rustls'] must be enabled"
);

#[cfg(any(
    all(feature = "_rt-actix", feature = "_rt-async-std"),
    all(feature = "_rt-actix", feature = "_rt-tokio"),
    all(feature = "_rt-async-std", feature = "_rt-tokio"),
    all(feature = "_tls-native-tls", feature = "_tls-rustls"),
))]
compile_error!(
    "only one of ['runtime-actix-native-tls', 'runtime-async-std-native-tls', \
     'runtime-tokio-native-tls', 'runtime-actix-rustls', 'runtime-async-std-rustls', \
     'runtime-tokio-rustls'] can be enabled"
);

#[cfg(feature = "_rt-async-std")]
mod rt_async_std;

#[cfg(any(feature = "_rt-tokio", feature = "_rt-actix"))]
mod rt_tokio;

#[cfg(all(feature = "_tls-native-tls"))]
pub use native_tls;

//
// Actix *OR* Tokio
//

#[cfg(all(any(feature = "_rt-tokio", feature = "_rt-actix"),))]
pub use rt_tokio::*;

#[cfg(all(
    feature = "_rt-async-std",
    not(any(feature = "_rt-tokio", feature = "_rt-actix"))
))]
pub use rt_async_std::*;
