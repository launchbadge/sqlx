use sqlx_core::error::Error;

pub(crate) fn put_length_prefixed<R>(
    buf: &mut Vec<u8>,
    f: impl FnOnce(&mut Vec<u8>) -> Result<R, Error>,
) -> Result<R, Error> {
    let offset = buf.len();
    buf.resize(offset + 4, 0);

    let r = f(buf)?;

    let len = (buf.len() - offset) as i32;
    (&mut buf[offset..offset + 4]).copy_from_slice(&len.to_be_bytes());

    Ok(r)
}

#[inline]
pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
    buf.push(b'\0');
}
