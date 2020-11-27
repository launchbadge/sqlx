mod socket;
mod tls;

pub use socket::Socket;
pub use tls::{CertificateInput, MaybeTlsStream};
