mod socket;

#[cfg(not(feature = "_rt-wasm-bindgen"))]
mod tls;

pub use socket::Socket;

#[cfg(not(feature = "_rt-wasm-bindgen"))]
pub use tls::{CertificateInput, MaybeTlsStream};

#[cfg(any(feature = "_rt-async-std", feature = "_rt-wasm-bindgen"))]
type PollReadBuf<'a> = [u8];

#[cfg(any(feature = "_rt-actix", feature = "_rt-tokio"))]
type PollReadBuf<'a> = sqlx_rt::ReadBuf<'a>;

#[cfg(any(feature = "_rt-async-std", feature = "_rt-wasm-bindgen"))]
type PollReadOut = usize;

#[cfg(any(feature = "_rt-actix", feature = "_rt-tokio"))]
type PollReadOut = ();
