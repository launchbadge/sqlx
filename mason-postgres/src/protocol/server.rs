use byteorder::{BigEndian, ByteOrder};
use bytes::{Bytes, BytesMut};
use std::{io, str};

// Reference
// https://www.postgresql.org/docs/devel/protocol-message-formats.html
// https://www.postgresql.org/docs/devel/protocol-message-types.html

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    /// Authentication was successful.
    AuthenticationOk,

    /// Authentication request for a cleartext password.
    AuthenticationCleartextPassword,

    /// Authentication request for an MD5-encrypted password.
    AuthenticationMd5Password(AuthenticationMd5Password),

    /// The client must save these values if it wishes to be able
    /// to issue CancelRequest messages later.
    BackendKeyData(BackendKeyData),

    BindComplete,
    CloseComplete,
    CommandComplete(CommandComplete),
    DataRow(DataRow),

    /// Response to an empty query string (substitutes for `CommandComplete`).
    EmptyQueryResponse,

    ErrorResponse(ErrorResponse),
    NoData,
    ParameterDescription(ParameterDescription),
    ParameterStatus(ParameterStatus),
    ParseComplete,
    PortalSuspended,
    ReadyForQuery(ReadyForQuery),
    RowDescription(RowDescription),
}

impl Message {
    pub fn deserialize(buf: &mut BytesMut) -> io::Result<Option<Self>> {
        if buf.len() < 5 {
            // No message is less than 5 bytes
            return Ok(None);
        }

        let tag = buf[0];

        if tag == 0 {
            panic!("handle graceful close");
        }

        // FIXME: What happens if len(u32) < len(usize) ?
        let len = BigEndian::read_u32(&buf[1..5]) as usize;

        if buf.len() < len + 1 {
            // Haven't received enough (yet)
            return Ok(None);
        }

        let buf = buf.split_to(len + 1).freeze();
        let idx = 5;

        Ok(Some(match tag {
            b'E' => Message::ErrorResponse(ErrorResponse { storage: buf.slice_from(idx) }),

            b'S' => {
                let name = read_str(buf.slice_from(idx))?;
                let value = read_str(buf.slice_from(idx + name.len() + 1))?;

                Message::ParameterStatus(ParameterStatus { name, value })
            }

            b'R' => match BigEndian::read_i32(&buf[idx..]) {
                0 => Message::AuthenticationOk,
                5 => Message::AuthenticationMd5Password(AuthenticationMd5Password {
                    salt: buf.slice_from(idx + 4),
                }),

                code => {
                    unimplemented!("unknown response code received: {:x}", code);
                }
            },

            b'K' => Message::BackendKeyData(BackendKeyData {
                process_id: BigEndian::read_i32(&buf[idx..]),
                secret_key: BigEndian::read_i32(&buf[(idx + 4)..]),
            }),

            b'Z' => Message::ReadyForQuery(ReadyForQuery { status: buf[idx] }),

            _ => unimplemented!("unknown tag received: {:x}", tag),
        }))
    }
}

#[derive(Debug)]
pub struct AuthenticationMd5Password {
    pub(super) salt: Bytes,
}

impl AuthenticationMd5Password {
    #[inline]
    pub fn salt(&self) -> &[u8] {
        &self.salt
    }
}

#[derive(Debug)]
pub struct DataRow {
    pub(super) storage: Bytes,
    pub(super) len: u16,
}

#[derive(Debug)]
pub struct BackendKeyData {
    pub(super) process_id: i32,
    pub(super) secret_key: i32,
}

impl BackendKeyData {
    #[inline]
    pub fn process_id(&self) -> i32 {
        self.process_id
    }

    #[inline]
    pub fn secret_key(&self) -> i32 {
        self.secret_key
    }
}

#[derive(Debug)]
pub struct CommandComplete {
    pub(super) tag: Bytes,
}

#[derive(Debug)]
pub struct ErrorResponse {
    pub(super) storage: Bytes,
}

#[derive(Debug)]
pub struct ParameterDescription {
    pub(super) storage: Bytes,
    pub(super) len: u16,
}

#[derive(Debug)]
pub struct ParameterStatus {
    pub(super) name: Bytes,
    pub(super) value: Bytes,
}

impl ParameterStatus {
    #[inline]
    pub fn name(&self) -> io::Result<&str> {
        Ok(str::from_utf8(&self.name).map_err(|_| io::ErrorKind::InvalidInput)?)
    }

    #[inline]
    pub fn value(&self) -> io::Result<&str> {
        Ok(str::from_utf8(&self.value).map_err(|_| io::ErrorKind::InvalidInput)?)
    }
}

#[derive(Debug)]
pub struct ReadyForQuery {
    pub(super) status: u8,
}

#[derive(Debug)]
pub struct RowDescription {
    pub(super) storage: Bytes,
    pub(super) len: u16,
}

#[inline]
fn read_str(buf: Bytes) -> io::Result<Bytes> {
    Ok(buf.slice_to(memchr::memchr(0, &buf).ok_or(io::ErrorKind::UnexpectedEof)?))
}
