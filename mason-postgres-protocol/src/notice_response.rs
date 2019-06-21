use crate::{decode::get_str, Decode, Encode};
use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use std::{
    fmt,
    io::{self, Write},
    pin::Pin,
    ptr::NonNull,
    str::{self, FromStr},
};
use std::borrow::Cow;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum Severity {
    Panic,
    Fatal,
    Error,
    Warning,
    Notice,
    Debug,
    Info,
    Log,
}

impl FromStr for Severity {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        Ok(match s {
            "PANIC" => Severity::Panic,
            "FATAL" => Severity::Fatal,
            "ERROR" => Severity::Error,
            "WARNING" => Severity::Warning,
            "NOTICE" => Severity::Notice,
            "DEBUG" => Severity::Debug,
            "INFO" => Severity::Info,
            "LOG" => Severity::Log,

            _ => {
                return Err(io::ErrorKind::InvalidData)?;
            }
        })
    }
}

// NOTE: `NoticeResponse` is lazily decoded on access to `.fields()`
#[derive(Clone)]
pub struct NoticeResponse(Bytes);

impl NoticeResponse {
    #[inline]
    pub fn fields(self) -> io::Result<NoticeResponseFields> { NoticeResponseFields::decode(self.0) }
}

impl fmt::Debug for NoticeResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Proxy format to the results of decoding the fields
        self.clone().fields().fmt(f)
    }
}

impl Encode for NoticeResponse {
    #[inline]
    fn size_hint(&self) -> usize { self.0.len() + 5 }

    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.write_u8(b'Z')?;
        buf.write_u32::<BigEndian>((4 + self.0.len()) as u32)?;
        buf.write_all(&self.0)?;

        Ok(())
    }
}

impl Decode for NoticeResponse {
    #[inline]
    fn decode(src: Bytes) -> io::Result<Self>
    where
        Self: Sized,
    {
        // NOTE: Further decoding is delayed until `.fields()`
        Ok(Self(src))
    }
}

pub struct NoticeResponseFields {
    #[used]
    storage: Pin<Bytes>,
    severity: Severity,
    code: NonNull<str>,
    message: NonNull<str>,
    detail: Option<NonNull<str>>,
    hint: Option<NonNull<str>>,
    position: Option<usize>,
    internal_position: Option<usize>,
    internal_query: Option<NonNull<str>>,
    where_: Option<NonNull<str>>,
    schema: Option<NonNull<str>>,
    table: Option<NonNull<str>>,
    column: Option<NonNull<str>>,
    data_type: Option<NonNull<str>>,
    constraint: Option<NonNull<str>>,
    file: Option<NonNull<str>>,
    line: Option<usize>,
    routine: Option<NonNull<str>>,
}

// SAFE: Raw pointers point to pinned memory inside the struct
unsafe impl Send for NoticeResponseFields {}
unsafe impl Sync for NoticeResponseFields {}

impl NoticeResponseFields {
    #[inline]
    pub fn severity(&self) -> Severity { self.severity }

    #[inline]
    pub fn code(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.code.as_ref() }
    }

    #[inline]
    pub fn message(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.message.as_ref() }
    }

    #[inline]
    pub fn detail(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.detail.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn hint(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.hint.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn position(&self) -> Option<usize> { self.position }

    #[inline]
    pub fn internal_position(&self) -> Option<usize> { self.internal_position }

    #[inline]
    pub fn internal_query(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.internal_query.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn where_(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.where_.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn schema(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.schema.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn table(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.table.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn column(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.column.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn data_type(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.data_type.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn constraint(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.constraint.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn file(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.file.as_ref().map(|s| s.as_ref()) }
    }

    #[inline]
    pub fn line(&self) -> Option<usize> { self.line }

    #[inline]
    pub fn routine(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.routine.as_ref().map(|s| s.as_ref()) }
    }
}

impl fmt::Debug for NoticeResponseFields {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NoticeResponseFields")
            .field("severity", &self.severity)
            .field("code", &self.code())
            .field("message", &self.message())
            .field("detail", &self.detail())
            .field("hint", &self.hint())
            .field("position", &self.position())
            .field("internal_position", &self.internal_position())
            .field("internal_query", &self.internal_query())
            .field("where_", &self.where_())
            .field("schema", &self.schema())
            .field("table", &self.table())
            .field("column", &self.column())
            .field("data_type", &self.data_type())
            .field("constraint", &self.constraint())
            .field("file", &self.file())
            .field("line", &self.line())
            .field("routine", &self.routine())
            .finish()
    }
}

impl Decode for NoticeResponseFields {
    fn decode(src: Bytes) -> io::Result<Self> {
        let storage = Pin::new(src);

        let mut code = None::<&str>;
        let mut message = None::<&str>;
        let mut severity = None::<&str>;
        let mut severity_non_local = None::<Severity>;
        let mut detail = None::<&str>;
        let mut hint = None::<&str>;
        let mut position = None::<usize>;
        let mut internal_position = None::<usize>;
        let mut internal_query = None::<&str>;
        let mut where_ = None::<&str>;
        let mut schema = None::<&str>;
        let mut table = None::<&str>;
        let mut column = None::<&str>;
        let mut data_type = None::<&str>;
        let mut constraint = None::<&str>;
        let mut file = None::<&str>;
        let mut line = None::<usize>;
        let mut routine = None::<&str>;

        let mut idx = 0;

        loop {
            let field_type = storage[idx];
            idx += 1;

            if field_type == 0 {
                break;
            }

            let field_value = get_str(&storage[idx..])?;
            idx += field_value.len() + 1;

            match field_type {
                b'S' => {
                    severity = Some(field_value);
                }

                b'V' => {
                    severity_non_local = Some(field_value.parse()?);
                }

                b'C' => {
                    code = Some(field_value);
                }

                b'M' => {
                    message = Some(field_value);
                }

                b'D' => {
                    detail = Some(field_value);
                }

                b'H' => {
                    hint = Some(field_value);
                }

                b'P' => {
                    position = Some(field_value.parse().map_err(|_| io::ErrorKind::InvalidData)?);
                }

                b'p' => {
                    internal_position =
                        Some(field_value.parse().map_err(|_| io::ErrorKind::InvalidData)?);
                }

                b'q' => {
                    internal_query = Some(field_value);
                }

                b'w' => {
                    where_ = Some(field_value);
                }

                b's' => {
                    schema = Some(field_value);
                }

                b't' => {
                    table = Some(field_value);
                }

                b'c' => {
                    column = Some(field_value);
                }

                b'd' => {
                    data_type = Some(field_value);
                }

                b'n' => {
                    constraint = Some(field_value);
                }

                b'F' => {
                    file = Some(field_value);
                }

                b'L' => {
                    line = Some(field_value.parse().map_err(|_| io::ErrorKind::InvalidData)?);
                }

                b'R' => {
                    routine = Some(field_value);
                }

                _ => {
                    unimplemented!(
                        "error/notice message field {:?} not implemented",
                        field_type as char
                    );
                }
            }
        }

        let severity = severity_non_local
            .or_else(move || severity?.parse().ok())
            .expect("required by protocol");

        let code = NonNull::from(code.expect("required by protocol"));
        let message = NonNull::from(message.expect("required by protocol"));
        let detail = detail.map(NonNull::from);
        let hint = hint.map(NonNull::from);
        let internal_query = internal_query.map(NonNull::from);
        let where_ = where_.map(NonNull::from);
        let schema = schema.map(NonNull::from);
        let table = table.map(NonNull::from);
        let column = column.map(NonNull::from);
        let data_type = data_type.map(NonNull::from);
        let constraint = constraint.map(NonNull::from);
        let file = file.map(NonNull::from);
        let routine = routine.map(NonNull::from);

        Ok(Self {
            storage,
            severity,
            code,
            message,
            detail,
            hint,
            internal_query,
            where_,
            schema,
            table,
            column,
            data_type,
            constraint,
            file,
            routine,
            line,
            position,
            internal_position,
        })
    }
}

pub struct NoticeResponseBuilder<'a> {
    severity: Severity,
    code: Cow<'a, str>,
    message: Cow<'a, str>,
    detail: Option<Cow<'a, str>>,
    hint: Option<Cow<'a, str>>,
    position: Option<usize>,
    internal_position: Option<usize>,
    internal_query: Option<Cow<'a, str>>,
    where_: Option<Cow<'a, str>>,
    schema: Option<Cow<'a, str>>,
    table: Option<Cow<'a, str>>,
    column: Option<Cow<'a, str>>,
    data_type: Option<Cow<'a, str>>,
    constraint: Option<Cow<'a, str>>,
    file: Option<Cow<'a, str>>,
    line: Option<usize>,
    routine: Option<Cow<'a, str>>,
}

impl Default for NoticeResponseBuilder<'_> {
    fn default() -> Self {
        Self {
            severity: Severity::Notice,
            message: Cow::Borrowed(""),
            code: Cow::Borrowed("XX000"),  // internal_error
            detail: None,
            hint: None,
            position: None,
            internal_position: None,
            internal_query: None,
            where_: None,
            schema: None,
            table: None,
            column: None,
            data_type: None,
            constraint: None,
            file: None,
            line: None,
            routine: None,
        }
    }
}

impl<'a> NoticeResponseBuilder<'a> {
    #[inline]
    pub fn new() -> NoticeResponseBuilder<'a> {
        Self::default()
    }

    #[inline]
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    #[inline]
    pub fn message(mut self, message: impl Into<Cow<'a, str>>) -> Self {
        self.message = message.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{Decode, Message, Severity};
    use bytes::Bytes;
    use std::io;

    #[test]
    fn it_decodes_notice_response() -> io::Result<()> {
        let src = Bytes::from_static(b"N\0\0\0pSNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0");
        let message = Message::decode(src)?;

        // FIXME: Is there a simpler pattern here for tests?
        let body = if let Message::NoticeResponse(body) = message {
            body
        } else {
            panic!("unexpected {:?}", message);
        };

        let fields = body.fields()?;

        assert_eq!(fields.severity(), Severity::Notice);
        assert_eq!(fields.message(), "extension \"uuid-ossp\" already exists, skipping");
        assert_eq!(fields.code(), "42710");
        assert_eq!(fields.file(), Some("extension.c"));
        assert_eq!(fields.line(), Some(1656));
        assert_eq!(fields.routine(), Some("CreateExtension"));

        Ok(())
    }
}
