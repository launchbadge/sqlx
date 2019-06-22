use crate::{decode::get_str, Decode, Encode};
use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use std::{
    fmt,
    io::{self, Write},
    ops::Range,
    pin::Pin,
    ptr::NonNull,
    str::{self, FromStr},
};

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

impl Severity {
    pub fn is_error(&self) -> bool {
        match self {
            Severity::Panic | Severity::Fatal | Severity::Error => true,
            _ => false,
        }
    }

    pub fn is_notice(&self) -> bool {
        match self {
            Severity::Warning
            | Severity::Notice
            | Severity::Debug
            | Severity::Info
            | Severity::Log => true,

            _ => false,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Severity::Panic => "PANIC",
            Severity::Fatal => "FATAL",
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
            Severity::Notice => "NOTICE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Log => "LOG",
        }
    }
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

#[derive(Clone)]
pub struct Response {
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
unsafe impl Send for Response {}
unsafe impl Sync for Response {}

impl Response {
    #[inline]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    #[inline]
    pub fn severity(&self) -> Severity {
        self.severity
    }

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
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    #[inline]
    pub fn internal_position(&self) -> Option<usize> {
        self.internal_position
    }

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
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    #[inline]
    pub fn routine(&self) -> Option<&str> {
        // SAFE: Memory is pinned
        unsafe { self.routine.as_ref().map(|s| s.as_ref()) }
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Response")
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

impl Encode for Response {
    #[inline]
    fn size_hint(&self) -> usize {
        self.storage.len() + 5
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        if self.severity.is_error() {
            buf.push(b'E');
        } else {
            buf.push(b'N');
        }

        buf.write_u32::<BigEndian>((4 + self.storage.len()) as u32)?;
        buf.write_all(&self.storage)?;

        Ok(())
    }
}

impl Decode for Response {
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
                        "response message field {:?} not implemented",
                        field_type as char
                    );
                }
            }
        }

        let severity = severity_non_local
            .or_else(move || severity?.parse().ok())
            .expect("`severity` required by protocol");

        let code = NonNull::from(code.expect("`code` required by protocol"));
        let message = NonNull::from(message.expect("`message` required by protocol"));
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

pub struct ResponseBuilder {
    storage: Vec<u8>,
    severity: Option<Severity>,
    code: Option<Range<usize>>,
    message: Option<Range<usize>>,
    detail: Option<Range<usize>>,
    hint: Option<Range<usize>>,
    position: Option<usize>,
    internal_position: Option<usize>,
    internal_query: Option<Range<usize>>,
    where_: Option<Range<usize>>,
    schema: Option<Range<usize>>,
    table: Option<Range<usize>>,
    column: Option<Range<usize>>,
    data_type: Option<Range<usize>>,
    constraint: Option<Range<usize>>,
    file: Option<Range<usize>>,
    line: Option<usize>,
    routine: Option<Range<usize>>,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            // FIXME: Remove this allocation (on the quest for zero-allocation)
            storage: Vec::with_capacity(128),
            severity: None,
            message: None,
            code: None,
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

fn put_str(buf: &mut Vec<u8>, tag: u8, value: impl AsRef<str>) -> Range<usize> {
    buf.push(tag);
    let beg = buf.len();
    buf.extend_from_slice(value.as_ref().as_bytes());
    let end = buf.len();
    buf.push(0);
    beg..end
}

impl ResponseBuilder {
    #[inline]
    pub fn new() -> ResponseBuilder {
        Self::default()
    }

    #[inline]
    pub fn severity(mut self, severity: Severity) -> Self {
        let sev = severity.to_str();

        let _ = put_str(&mut self.storage, b'S', sev);
        let _ = put_str(&mut self.storage, b'V', sev);

        self.severity = Some(severity);
        self
    }

    #[inline]
    pub fn message(mut self, message: impl AsRef<str>) -> Self {
        self.message = Some(put_str(&mut self.storage, b'M', message));
        self
    }

    #[inline]
    pub fn code(mut self, code: impl AsRef<str>) -> Self {
        self.code = Some(put_str(&mut self.storage, b'C', code));
        self
    }

    #[inline]
    pub fn detail(mut self, detail: impl AsRef<str>) -> Self {
        self.detail = Some(put_str(&mut self.storage, b'D', detail));
        self
    }

    #[inline]
    pub fn hint(mut self, hint: impl AsRef<str>) -> Self {
        self.hint = Some(put_str(&mut self.storage, b'H', hint));
        self
    }

    #[inline]
    pub fn position(mut self, position: usize) -> Self {
        self.storage.push(b'P');
        // PANIC: Write to Vec<u8> is infallible
        itoa::write(&mut self.storage, position).unwrap();
        self.storage.push(0);

        self.position = Some(position);
        self
    }

    #[inline]
    pub fn internal_position(mut self, position: usize) -> Self {
        self.storage.push(b'p');
        // PANIC: Write to Vec<u8> is infallible
        itoa::write(&mut self.storage, position).unwrap();
        self.storage.push(0);

        self.internal_position = Some(position);
        self
    }

    #[inline]
    pub fn internal_query(mut self, query: impl AsRef<str>) -> Self {
        self.internal_query = Some(put_str(&mut self.storage, b'q', query));
        self
    }

    #[inline]
    pub fn where_(mut self, where_: impl AsRef<str>) -> Self {
        self.where_ = Some(put_str(&mut self.storage, b'w', where_));
        self
    }

    #[inline]
    pub fn schema(mut self, schema: impl AsRef<str>) -> Self {
        self.schema = Some(put_str(&mut self.storage, b's', schema));
        self
    }

    #[inline]
    pub fn table(mut self, table: impl AsRef<str>) -> Self {
        self.table = Some(put_str(&mut self.storage, b't', table));
        self
    }

    #[inline]
    pub fn column(mut self, column: impl AsRef<str>) -> Self {
        self.column = Some(put_str(&mut self.storage, b'c', column));
        self
    }

    #[inline]
    pub fn data_type(mut self, data_type: impl AsRef<str>) -> Self {
        self.data_type = Some(put_str(&mut self.storage, b'd', data_type));
        self
    }

    #[inline]
    pub fn constraint(mut self, constraint: impl AsRef<str>) -> Self {
        self.constraint = Some(put_str(&mut self.storage, b'n', constraint));
        self
    }

    #[inline]
    pub fn file(mut self, file: impl AsRef<str>) -> Self {
        self.file = Some(put_str(&mut self.storage, b'F', file));
        self
    }

    #[inline]
    pub fn line(mut self, line: usize) -> Self {
        self.storage.push(b'L');
        // PANIC: Write to Vec<u8> is infallible
        itoa::write(&mut self.storage, line).unwrap();
        self.storage.push(0);

        self.line = Some(line);
        self
    }

    #[inline]
    pub fn routine(mut self, routine: impl AsRef<str>) -> Self {
        self.routine = Some(put_str(&mut self.storage, b'R', routine));
        self
    }

    pub fn build(mut self) -> Response {
        // Add a \0 terminator
        self.storage.push(0);

        // Freeze the storage and Pin so we can self-reference it
        let storage = Pin::new(Bytes::from(self.storage));

        let make_str_ref = |val: Option<Range<usize>>| unsafe {
            val.map(|r| NonNull::from(str::from_utf8_unchecked(&storage[r])))
        };

        let code = make_str_ref(self.code);
        let message = make_str_ref(self.message);
        let detail = make_str_ref(self.detail);
        let hint = make_str_ref(self.hint);
        let internal_query = make_str_ref(self.internal_query);
        let where_ = make_str_ref(self.where_);
        let schema = make_str_ref(self.schema);
        let table = make_str_ref(self.table);
        let column = make_str_ref(self.column);
        let data_type = make_str_ref(self.data_type);
        let constraint = make_str_ref(self.constraint);
        let file = make_str_ref(self.file);
        let routine = make_str_ref(self.routine);

        Response {
            storage,
            // FIXME: Default and don't panic here
            severity: self.severity.expect("`severity` required by protocol"),
            code: code.expect("`code` required by protocol"),
            message: message.expect("`message` required by protocol"),
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
            line: self.line,
            position: self.position,
            internal_position: self.internal_position,
        }
    }
}

impl Encode for ResponseBuilder {
    #[inline]
    fn size_hint(&self) -> usize {
        self.storage.len() + 6
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        if self.severity.as_ref().map_or(false, |s| s.is_error()) {
            buf.push(b'E');
        } else {
            buf.push(b'N');
        }

        buf.write_u32::<BigEndian>((5 + self.storage.len()) as u32)?;
        buf.write_all(&self.storage)?;
        buf.push(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, Severity};
    use crate::{Decode, Encode};
    use bytes::Bytes;
    use std::io;

    const RESPONSE: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, \
          skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    #[test]
    fn it_encodes_response() -> io::Result<()> {
        let message = Response::builder()
            .severity(Severity::Notice)
            .code("42710")
            .message("extension \"uuid-ossp\" already exists, skipping")
            .file("extension.c")
            .line(1656)
            .routine("CreateExtension")
            .build();

        let mut dst = Vec::with_capacity(message.size_hint());
        message.encode(&mut dst)?;

        assert_eq!(&dst[5..], RESPONSE);

        Ok(())
    }

    #[test]
    fn it_encodes_response_builder() -> io::Result<()> {
        let message = Response::builder()
            .severity(Severity::Notice)
            .code("42710")
            .message("extension \"uuid-ossp\" already exists, skipping")
            .file("extension.c")
            .line(1656)
            .routine("CreateExtension");

        let mut dst = Vec::with_capacity(message.size_hint());
        message.encode(&mut dst)?;

        assert_eq!(&dst[5..], RESPONSE);

        Ok(())
    }

    #[test]
    fn it_decodes_response() -> io::Result<()> {
        let src = Bytes::from_static(RESPONSE);
        let message = Response::decode(src)?;

        assert_eq!(message.severity(), Severity::Notice);
        assert_eq!(message.message(), "extension \"uuid-ossp\" already exists, skipping");
        assert_eq!(message.code(), "42710");
        assert_eq!(message.file(), Some("extension.c"));
        assert_eq!(message.line(), Some(1656));
        assert_eq!(message.routine(), Some("CreateExtension"));

        Ok(())
    }
}
