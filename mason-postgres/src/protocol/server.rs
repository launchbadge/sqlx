use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use bytes::{Bytes, BytesMut};
use memchr::memchr;
use std::{io, str};

// FIXME: This should probably move up
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Not in a transaction block.
    Idle,

    /// In a transaction block.
    Transaction,

    /// In a _failed_ transaction block. Queries will be rejected until block is ended.
    Error,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            b'I' => Status::Idle,
            b'T' => Status::Transaction,
            b'E' => Status::Error,

            _ => unreachable!(),
        }
    }
}

// FIXME: This should probably move up
#[derive(Debug, PartialEq)]
pub enum Format {
    Text,
    Binary,
}

impl From<u16> for Format {
    fn from(value: u16) -> Self {
        match value {
            0 => Format::Text,
            1 => Format::Binary,

            _ => unreachable!(),
        }
    }
}

// Reference
// https://www.postgresql.org/docs/devel/protocol-message-formats.html
// https://www.postgresql.org/docs/devel/protocol-message-types.html

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    /// Authentication was successful.
    AuthenticationOk,

    /// Specifies that Kerberos V5 authentication is required.
    #[deprecated]
    AuthenticationKerberosV5,

    /// Specifies that a clear-text password is required.
    AuthenticationClearTextPassword,

    /// Specifies that an MD5-encrypted password is required.
    AuthenticationMd5Password(AuthenticationMd5Password),

    /// Specifies that an SCM credentials message is required.
    AuthenticationScmCredential,

    /// Specifies that GSSAPI authentication is required.
    AuthenticationGss,

    /// Specifies that SSPI authentication is required.
    AuthenticationSspi,

    /// Specifies that this message contains GSSAPI or SSPI data.
    AuthenticationGssContinue(AuthenticationGssContinue),

    /// Specifies that SASL authentication is required.
    AuthenticationSasl(AuthenticationSasl),

    /// Specifies that this message contains a SASL challenge.
    AuthenticationSaslContinue(AuthenticationSaslContinue),

    /// Specifies that SASL authentication has completed.
    AuthenticationSaslFinal(AuthenticationSaslFinal),

    /// Identifies the message as cancellation key data.
    /// The client must save these values if it wishes to be
    /// able to issue CancelRequest messages later.
    BackendKeyData(BackendKeyData),

    /// Identifies the message as a Bind-complete indicator.
    BindComplete,

    /// Identifies the message as a Close-complete indicator.
    CloseComplete,

    /// Identifies the message as a command-completed response.
    CommandComplete(CommandComplete),

    /// Identifies the message as COPY data.
    CopyData(CopyData),

    /// Identifies the message as a COPY-complete indicator.
    CopyDone,

    /// Identifies the message as a Start Copy In response.
    /// The client must now send copy-in data (if not prepared to do so, send a CopyFail message).
    CopyInResponse(CopyResponse),

    /// Identifies the message as a Start Copy Out response. This message will be followed by copy-out data.
    CopyOutResponse(CopyResponse),

    /// Identifies the message as a Start Copy Both response.
    /// This message is used only for Streaming Replication.
    CopyBothResponse(CopyResponse),

    /// Identifies the message as a data row.
    DataRow(DataRow),

    /// Identifies the message as a response to an empty query string. (This substitutes for CommandComplete.)
    EmptyQueryResponse,

    /// Identifies the message as an error.
    ErrorResponse(ErrorResponse),

    /// Identifies the message as a protocol version negotiation message.
    NegotiateProtocolVersion(NegotiateProtocolVersion),

    /// Identifies the message as a no-data indicator.
    NoData,

    /// Identifies the message as a notice.
    NoticeResponse(NoticeResponse),

    /// Identifies the message as a notification response.
    NotificationResponse(NotificationResponse),

    /// Identifies the message as a parameter description.
    ParameterDescription(ParameterDescription),

    /// Identifies the message as a run-time parameter status report.
    ParameterStatus(ParameterStatus),

    /// Identifies the message as a Parse-complete indicator.
    ParseComplete,

    /// Identifies the message as a portal-suspended indicator. Note this only appears if an
    /// Execute message's row-count limit was reached.
    PortalSuspended,

    /// Identifies the message type. ReadyForQuery is sent whenever the backend is
    /// ready for a new query cycle.
    ReadyForQuery(ReadyForQuery),

    /// Identifies the message as a row description.
    RowDescription(RowDescription),
}

impl Message {
    // FIXME: Clean this up and do some benchmarking for performance
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

        log::trace!("recv: {:?}", buf);

        Ok(Some(match tag {
            // [x] AuthenticationOk
            // [x] AuthenticationKerberosV5
            // [x] AuthenticationClearTextPassword
            // [x] AuthenticationMd5Password(AuthenticationMd5Password)
            // [x] AuthenticationScmCredential
            // [x] AuthenticationSspi
            // [x] AuthenticationGssContinue(AuthenticationGssContinue)
            // [x] AuthenticationSasl(AuthenticationSasl)
            // [x] AuthenticationSaslContinue(AuthenticationSaslContinue)
            // [x] AuthenticationSaslFinal(AuthenticationSaslFinal)
            b'R' => match BigEndian::read_i32(&buf[idx..]) {
                0 => Message::AuthenticationOk,

                #[allow(deprecated)]
                2 => Message::AuthenticationKerberosV5,

                3 => Message::AuthenticationClearTextPassword,

                6 => Message::AuthenticationScmCredential,

                5 => Message::AuthenticationMd5Password(AuthenticationMd5Password {
                    salt: buf.slice_from(idx + 4),
                }),

                7 => Message::AuthenticationGss,

                9 => Message::AuthenticationSspi,

                8 => Message::AuthenticationGssContinue(AuthenticationGssContinue(
                    buf.slice_from(idx + 4),
                )),

                10 => Message::AuthenticationSasl(AuthenticationSasl(buf.slice_from(idx + 4))),

                11 => Message::AuthenticationSaslContinue(AuthenticationSaslContinue(
                    buf.slice_from(idx + 4),
                )),

                12 => Message::AuthenticationSaslFinal(AuthenticationSaslFinal(
                    buf.slice_from(idx + 4),
                )),

                code => {
                    unimplemented!("unknown authentication type received: {:x}", code);
                }
            },

            // [x] BackendKeyData(BackendKeyData)
            b'K' => Message::BackendKeyData(BackendKeyData {
                process_id: BigEndian::read_i32(&buf[idx..]),
                secret_key: BigEndian::read_i32(&buf[(idx + 4)..]),
            }),

            // [x] BindComplete
            b'2' => Message::BindComplete,

            // [x] CloseComplete
            b'3' => Message::CloseComplete,

            // [x] CommandComplete(CommandComplete)
            b'C' => Message::CommandComplete(CommandComplete { tag: buf.slice_from(idx) }),

            // [ ] CopyData(CopyData)

            // [x] CopyDone
            b'c' => Message::CopyDone,

            // [ ] CopyInResponse(CopyInResponse)
            // [ ] CopyOutResponse(CopyOutResponse)
            // [ ] CopyBothResponse(CopyBothResponse)
            // [ ] DataRow(DataRow)

            // [x] EmptyQueryResponse
            b'I' => Message::EmptyQueryResponse,

            // [ ] ErrorResponse(ErrorResponse)
            // [ ] NegotiateProtocolVersion(NegotiateProtocolVersion)

            // [x] NoData
            b'n' => Message::NoData,

            // [ ] NoticeResponse(NoticeResponse)
            b'N' => Message::NoticeResponse(NoticeResponse(buf.slice_from(idx))),

            // [ ] NotificationResponse(NotificationResponse)

            // [ ] ParameterDescription(ParameterDescription)

            // [x] ParameterStatus(ParameterStatus)
            b'S' => {
                let name = find_cstr(buf.slice_from(idx))?;
                let value = find_cstr(buf.slice_from(idx + name.len() + 1))?;

                Message::ParameterStatus(ParameterStatus { name, value })
            }

            // [x] ParseComplete
            b'1' => Message::ParseComplete,

            // [x] PortalSuspended
            b's' => Message::PortalSuspended,

            // [x] ReadyForQuery(ReadyForQuery)
            b'Z' => Message::ReadyForQuery(ReadyForQuery { status: Status::from(buf[idx]) }),

            // [ ] RowDescription(RowDescription)
            _ => unimplemented!("unknown tag received: {:x}", tag),
        }))
    }
}

// [-] AuthenticationOk
// [-] AuthenticationKerberosV5
// [-] AuthenticationClearTextPassword

// [x] AuthenticationMd5Password(AuthenticationMd5Password)

#[derive(Debug)]
pub struct AuthenticationMd5Password {
    /// The salt to use when encrypting the password.
    pub salt: Bytes,
}

// [-] AuthenticationScmCredential
// [-] AuthenticationSspi

// [x] AuthenticationGssContinue(AuthenticationGssContinue)

#[derive(Debug)]
pub struct AuthenticationGssContinue(pub Bytes);

// [x] AuthenticationSasl(AuthenticationSasl)

#[derive(Debug)]
pub struct AuthenticationSasl(pub Bytes);

impl AuthenticationSasl {
    #[inline]
    pub fn mechanisms(&self) -> SaslAuthenticationMechanisms<'_> {
        SaslAuthenticationMechanisms(&self.0)
    }
}

pub struct SaslAuthenticationMechanisms<'a>(&'a [u8]);

impl<'a> Iterator for SaslAuthenticationMechanisms<'a> {
    type Item = io::Result<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        let storage = self.0;
        memchr(0, storage)
            .ok_or_else(|| io::ErrorKind::UnexpectedEof.into())
            .and_then(|end| {
                if end == 0 {
                    // A zero byte is required as terminator after the
                    // last authentication mechanism name.
                    Ok(None)
                } else {
                    // Advance the storage reference to continue iteration
                    self.0 = &storage[end..];
                    to_str(&storage[..end]).map(Some)
                }
            })
            .transpose()
    }
}

// [x] AuthenticationSaslContinue(AuthenticationSaslContinue)

#[derive(Debug)]
pub struct AuthenticationSaslContinue(pub Bytes);

// [x] AuthenticationSaslFinal(AuthenticationSaslFinal)

#[derive(Debug)]
pub struct AuthenticationSaslFinal(pub Bytes);

// [x] BackendKeyData(BackendKeyData)

#[derive(Debug)]
pub struct BackendKeyData {
    pub process_id: i32,
    pub secret_key: i32,
}

// [-] BindComplete
// [-] CloseComplete

// [x] CommandComplete(CommandComplete)

#[derive(Debug)]
pub struct CommandComplete {
    pub tag: Bytes,
}

impl CommandComplete {
    #[inline]
    pub fn tag(&self) -> io::Result<&str> {
        to_str(&self.tag)
    }
}

// [x] CopyData(CopyData)

#[derive(Debug)]
pub struct CopyData(pub Bytes);

// [-] CopyDone

// [x] CopyInResponse(CopyResponse)
// [x] CopyOutResponse(CopyResponse)
// [x] CopyBothResponse(CopyResponse)

#[derive(Debug)]
pub struct CopyResponse {
    storage: Bytes,

    /// Indicates the "overall" `COPY` format.
    #[deprecated]
    pub format: Format,

    /// The number of columns in the data to be copied.
    pub columns: u16,
}

impl CopyResponse {
    /// The format for each column in the data to be copied.
    pub fn column_formats(&self) -> Formats<'_> {
        Formats { storage: &self.storage, remaining: self.columns }
    }
}

pub struct Formats<'a> {
    storage: &'a [u8],
    remaining: u16,
}

impl<'a> Iterator for Formats<'a> {
    type Item = io::Result<Format>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;
        self.storage.read_u16::<BigEndian>().map(Into::into).map(Some).transpose()
    }
}

// [ ] DataRow(DataRow)

#[derive(Debug)]
pub struct DataRow {
    storage: Bytes,
    len: u16,
}

// [-] EmptyQueryResponse

// [ ] ErrorResponse(ErrorResponse)

#[derive(Debug)]
pub struct ErrorResponse(pub Bytes);

impl ErrorResponse {
    #[inline]
    pub fn fields(&self) -> MessageFields<'_> {
        MessageFields(&self.0)
    }
}

// [ ] NegotiateProtocolVersion(NegotiateProtocolVersion)

#[derive(Debug)]
pub struct NegotiateProtocolVersion {}

// [-] NoData

// [x] NoticeResponse(NoticeResponse)

#[derive(Debug)]
pub struct NoticeResponse(pub Bytes);

impl NoticeResponse {
    #[inline]
    pub fn fields(&self) -> MessageFields<'_> {
        MessageFields(&self.0)
    }
}

#[derive(Debug)]
pub enum MessageField<'a> {
    /// The field contents are ERROR, FATAL, or PANIC (in an error message), or
    /// WARNING, NOTICE, DEBUG, INFO, or LOG (in a notice message), or a localized translation
    /// of one of these. Always present.
    Severity(&'a str),

    /// A non-localized version of `Severity`. Always present in PostgreSQL 9.6 or later.
    SeverityNonLocal(&'a str),

    /// The SQLSTATE code for the error. Always present.
    Code(&'a str),

    /// The primary human-readable error message.
    /// This should be accurate but terse (typically one line). Always present.
    Message(&'a str),

    /// An optional secondary error message carrying more detail about the problem.
    /// Might run to multiple lines.
    Detail(&'a str),

    /// An optional suggestion what to do about the problem. This is intended to differ from
    /// Detail in that it offers advice (potentially inappropriate) rather than hard facts.
    /// Might run to multiple lines.
    Hint(&'a str),

    /// Indicating an error cursor position as an index into the original query string.
    /// The first character has index 1, and positions are measured in characters not bytes.
    Position(usize),

    /// This is defined the same as the `Position` field, but it is used when the cursor
    /// position refers to an internally generated command rather than the
    /// one submitted by the client.
    ///
    /// The `InternalQuery` field will always appear when this field appears.
    InternalPosition(usize),

    /// The text of a failed internally-generated command. This could be,
    /// for example, a SQL query issued by a PL/pgSQL function.
    InternalQuery(&'a str),

    /// An indication of the context in which the error occurred. Presently this includes
    /// a call stack traceback of active procedural language functions and
    /// internally-generated queries. The trace is one entry per line, most recent first.
    Where(&'a str),

    /// If the error was associated with a specific database object, the name of
    /// the schema containing that object, if any.
    Schema(&'a str),

    /// If the error was associated with a specific table, the name of the table.
    Table(&'a str),

    /// If the error was associated with a specific table column, the name of the column.
    Column(&'a str),

    /// If the error was associated with a specific data type, the name of the data type.
    DataType(&'a str),

    /// If the error was associated with a specific constraint, the name of the constraint.
    Constraint(&'a str),

    /// The file name of the source-code location where the error was reported.
    File(&'a str),

    /// The line number of the source-code location where the error was reported.
    Line(usize),

    /// The name of the source-code routine reporting the error.
    Routine(&'a str),

    Unknown(u8, &'a str),
}

pub struct MessageFields<'a>(&'a [u8]);

impl<'a> Iterator for MessageFields<'a> {
    type Item = io::Result<MessageField<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .read_u8()
            .and_then(|token| {
                if token == 0 {
                    // End of fields
                    return Ok(None);
                }

                let end = find_nul(self.0)?;
                let value = to_str(&self.0[..end])?;
                self.0 = &self.0[(end + 1)..];

                Ok(Some(match token {
                    b'S' => MessageField::Severity(value),
                    b'V' => MessageField::SeverityNonLocal(value),
                    b'C' => MessageField::Code(value),
                    b'M' => MessageField::Message(value),
                    b'D' => MessageField::Detail(value),
                    b'H' => MessageField::Hint(value),
                    b'P' => MessageField::Position(to_usize(value)?),
                    b'p' => MessageField::InternalPosition(to_usize(value)?),
                    b'q' => MessageField::InternalQuery(value),
                    b'w' => MessageField::Where(value),
                    b's' => MessageField::Schema(value),
                    b't' => MessageField::Table(value),
                    b'c' => MessageField::Column(value),
                    b'd' => MessageField::DataType(value),
                    b'n' => MessageField::Constraint(value),
                    b'F' => MessageField::File(value),
                    b'L' => MessageField::Line(to_usize(value)?),
                    b'R' => MessageField::Routine(value),

                    _ => MessageField::Unknown(token, value),
                }))
            })
            .transpose()
    }
}

// [ ] NotificationResponse(NotificationResponse)

#[derive(Debug)]
pub struct NotificationResponse {}

// [ ] ParameterDescription(ParameterDescription)

#[derive(Debug)]
pub struct ParameterDescription {
    storage: Bytes,
    len: u16,
}

// [ ] ParameterStatus(ParameterStatus)

#[derive(Debug)]
pub struct ParameterStatus {
    name: Bytes,
    value: Bytes,
}

impl ParameterStatus {
    #[inline]
    pub fn name(&self) -> io::Result<&str> {
        to_str(&self.name)
    }

    #[inline]
    pub fn value(&self) -> io::Result<&str> {
        to_str(&self.value)
    }
}

// [-] ParseComplete
// [-] PortalSuspended

// [x] ReadyForQuery(ReadyForQuery)

#[derive(Debug)]
pub struct ReadyForQuery {
    pub status: Status,
}

// [ ] RowDescription(RowDescription)

#[derive(Debug)]
pub struct RowDescription {
    storage: Bytes,
    len: u16,
}

// ---

#[inline]
fn find_nul(b: &[u8]) -> io::Result<usize> {
    Ok(memchr(0, &b).ok_or(io::ErrorKind::UnexpectedEof)?)
}

#[inline]
fn find_cstr(b: Bytes) -> io::Result<Bytes> {
    Ok(b.slice_to(find_nul(&b)?))
}

#[inline]
fn to_str(b: &[u8]) -> io::Result<&str> {
    Ok(str::from_utf8(b).map_err(|_| io::ErrorKind::InvalidInput)?)
}

#[inline]
fn to_usize(s: &str) -> io::Result<usize> {
    Ok(s.parse().map_err(|_| io::ErrorKind::InvalidInput)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::assert_matches;
    use std::error::Error;

    // [x] AuthenticationOk
    // [ ] AuthenticationKerberosV5
    // [x] AuthenticationClearTextPassword
    // [x] AuthenticationMd5Password(AuthenticationMd5Password)
    // [ ] AuthenticationScmCredential
    // [ ] AuthenticationSspi
    // [ ] AuthenticationGssContinue(AuthenticationGssContinue)
    // [ ] AuthenticationSasl(AuthenticationSasl)
    // [ ] AuthenticationSaslContinue(AuthenticationSaslContinue)
    // [ ] AuthenticationSaslFinal(AuthenticationSaslFinal)

    #[test]
    fn it_decodes_authentication_ok() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"R\0\0\0\x08\0\0\0\0"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::AuthenticationOk));
        assert!(buf.is_empty());

        Ok(())
    }

    #[test]
    fn it_decodes_authentication_clear_text_password() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"R\0\0\0\x08\0\0\0\x03"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::AuthenticationClearTextPassword));
        assert!(buf.is_empty());

        Ok(())
    }

    #[test]
    fn it_decodes_authentication_md5_password() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"R\0\0\0\x0c\0\0\0\x05\x98\x153*"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::AuthenticationMd5Password(_)));
        assert!(buf.is_empty());

        if let Some(Message::AuthenticationMd5Password(body)) = msg {
            assert_eq!(&*body.salt, &[152, 21, 51, 42]);
        }

        Ok(())
    }

    // [ ] BackendKeyData(BackendKeyData)

    #[test]
    fn it_decodes_backend_key_data() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"K\0\0\0\x0c\0\0\x06\0_\x0f`a"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::BackendKeyData(_)));
        assert!(buf.is_empty());

        if let Some(Message::BackendKeyData(body)) = msg {
            assert_eq!(body.process_id, 1536);
            assert_eq!(body.secret_key, 0x5f0f6061);
        }

        Ok(())
    }

    // [ ] BindComplete
    // [ ] CloseComplete

    // [ ] CommandComplete(CommandComplete)

    #[test]
    fn it_decodes_command_complete() -> Result<(), Box<Error>> {
        // "C\0\0\0\x15CREATE EXTENSION\0"
        // TODO

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_with_rows_affected() -> Result<(), Box<Error>> {
        // TODO

        Ok(())
    }

    // [ ] CopyData(CopyData)
    // [ ] CopyDone
    // [ ] CopyInResponse(CopyInResponse)
    // [ ] CopyOutResponse(CopyOutResponse)
    // [ ] CopyBothResponse(CopyBothResponse)
    // [ ] DataRow(DataRow)
    // [ ] EmptyQueryResponse
    // [ ] ErrorResponse(ErrorResponse)
    // [ ] NegotiateProtocolVersion(NegotiateProtocolVersion)
    // [ ] NoData

    // [ ] NoticeResponse(NoticeResponse)

    #[test]
    fn it_decodes_notice_response() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"N\0\0\0pSNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::NoticeResponse(_)));
        assert!(buf.is_empty());

        if let Some(Message::NoticeResponse(body)) = msg {
            let fields = body.fields().collect::<Result<Vec<_>, _>>()?;

            assert_eq!(fields.len(), 7);
            assert_matches!(fields[0], MessageField::Severity("NOTICE"));
            assert_matches!(fields[1], MessageField::SeverityNonLocal("NOTICE"));
            assert_matches!(fields[2], MessageField::Code("42710"));
            assert_matches!(
                fields[3],
                MessageField::Message("extension \"uuid-ossp\" already exists, skipping")
            );
            assert_matches!(fields[4], MessageField::File("extension.c"));
            assert_matches!(fields[5], MessageField::Line(1656));
            assert_matches!(fields[6], MessageField::Routine("CreateExtension"));
        }

        Ok(())
    }

    // [ ] NotificationResponse(NotificationResponse)
    // [ ] ParameterDescription(ParameterDescription)

    // [x] ParameterStatus(ParameterStatus)

    #[test]
    fn it_decodes_parameter_status() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&"S\0\0\0\x19server_encoding\0UTF8\0"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::ParameterStatus(_)));
        assert!(buf.is_empty());

        if let Some(Message::ParameterStatus(body)) = msg {
            assert_eq!(body.name()?, "server_encoding");
            assert_eq!(body.value()?, "UTF8");
        }

        Ok(())
    }

    // [ ] ParseComplete
    // [ ] PortalSuspended

    // [x] ReadyForQuery(ReadyForQuery)

    #[test]
    fn it_decodes_ready_for_query() -> Result<(), Box<Error>> {
        let mut buf = BytesMut::from(&b"Z\0\0\0\x05I"[..]);
        let msg = Message::deserialize(&mut buf)?;

        assert_matches!(msg, Some(Message::ReadyForQuery(_)));
        assert!(buf.is_empty());

        if let Some(Message::ReadyForQuery(body)) = msg {
            assert_eq!(body.status, Status::Idle);
        }

        Ok(())
    }

    // [ ] RowDescription(RowDescription)
}
