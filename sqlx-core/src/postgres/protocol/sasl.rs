use crate::io::BufMut;
use crate::postgres::protocol::Write;
use crate::postgres::Postgres;
use byteorder::NetworkEndian;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub(crate) struct SaslInitialResponse<'a>(pub(crate) &'a str);

impl<'a> Write for SaslInitialResponse<'a> {
    fn write(&self, buf: &mut Vec<u8>) {
        let len = self.0.as_bytes().len() as u32;
        buf.push(b'p');
        buf.put_u32::<NetworkEndian>(4u32 + len + 14u32 + 4u32);
        buf.put_str_nul("SCRAM-SHA-256");
        buf.put_u32::<NetworkEndian>(len);
        buf.extend_from_slice(self.0.as_bytes());
    }
}

pub(crate) struct SaslResponse<'a>(pub(crate) &'a str);

impl<'a> Write for SaslResponse<'a> {
    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(b'p');
        buf.put_u32::<NetworkEndian>(4u32 + self.0.as_bytes().len() as u32);
        buf.extend_from_slice(self.0.as_bytes());
    }
}

// Hi(str, salt, i):
pub(crate) fn hi<'a>(
    s: &'a str,
    salt: &'a [u8],
    iter_count: u32,
) -> crate::Result<Postgres, [u8; 32]> {
    let mut mac = Hmac::<Sha256>::new_varkey(s.as_bytes())
        .map_err(|_| protocol_err!("HMAC can take key of any size"))?;

    mac.input(&salt);
    mac.input(&1u32.to_be_bytes());

    let mut u = mac.result().code();
    let mut hi = u;

    for _ in 1..iter_count {
        let mut mac = Hmac::<Sha256>::new_varkey(s.as_bytes())
            .map_err(|_| protocol_err!("HMAC can take key of any size"))?;
        mac.input(u.as_slice());
        u = mac.result().code();
        hi = hi.iter().zip(u.iter()).map(|(&a, &b)| a ^ b).collect();
    }

    Ok(hi.into())
}
