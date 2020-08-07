use bytes::{Buf, Bytes};
use memchr::memchr;
use sqlx_core::error::Error;

pub(crate) fn get_str_bytes(buf: &mut Bytes) -> Result<Bytes, Error> {
    let nul = memchr(b'\0', buf.bytes())
        .ok_or_else(|| Error::protocol_msg("expected NUL in byte sequence"))?;

    let v = buf.slice(0..nul);

    buf.advance(nul + 1);

    Ok(v)
}

pub(crate) fn put_length_prefixed<R>(
    buf: &mut Vec<u8>,
    inclusive: bool,
    f: impl FnOnce(&mut Vec<u8>) -> Result<R, Error>,
) -> Result<R, Error> {
    let offset = buf.len();
    buf.resize(offset + 4, 0);

    let r = f(buf)?;

    let mut len = (buf.len() - offset) as i32;

    if !inclusive {
        len -= 4;
    }

    (&mut buf[offset..offset + 4]).copy_from_slice(&len.to_be_bytes());

    Ok(r)
}

pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
    buf.push(b'\0');
}

// writes a statement name by ID
pub(crate) fn put_statement_name(buf: &mut Vec<u8>, id: Option<u32>) {
    if let Some(id) = id {
        buf.extend(b"sqlx_s_");

        itoa::write(&mut *buf, id).unwrap();
    }

    buf.push(0);
}

// writes a portal name by ID
pub(crate) fn put_portal_name(buf: &mut Vec<u8>, id: Option<u32>) {
    if let Some(id) = id {
        buf.extend(b"sqlx_p_");

        itoa::write(&mut *buf, id).unwrap();
    }

    buf.push(0);
}
