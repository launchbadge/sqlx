use super::{decode::get_str, Decode};
use bytes::Bytes;
use std::{
    fmt, io,
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
    pub fn is_error(self) -> bool {
        match self {
            Severity::Panic | Severity::Fatal | Severity::Error => true,
            _ => false,
        }
    }

    pub fn is_notice(self) -> bool {
        match self {
            Severity::Warning
            | Severity::Notice
            | Severity::Debug
            | Severity::Info
            | Severity::Log => true,

            _ => false,
        }
    }

    pub fn to_str(self) -> &'static str {
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
    storage: Pin<Vec<u8>>,
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

impl Decode for Response {
    fn decode(src: &[u8]) -> io::Result<Self> {
        let storage = Pin::new(Vec::from(src));

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
                    position = Some(
                        field_value
                            .parse()
                            .map_err(|_| io::ErrorKind::InvalidData)?,
                    );
                }

                b'p' => {
                    internal_position = Some(
                        field_value
                            .parse()
                            .map_err(|_| io::ErrorKind::InvalidData)?,
                    );
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
                    line = Some(
                        field_value
                            .parse()
                            .map_err(|_| io::ErrorKind::InvalidData)?,
                    );
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

#[cfg(test)]
mod tests {
    use super::{Decode, Response, Severity};
    use bytes::Bytes;
    use std::io;

    const RESPONSE: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, \
          skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    #[test]
    fn it_decodes_response() -> io::Result<()> {
        let message = Response::decode(RESPONSE)?;

        assert_eq!(message.severity(), Severity::Notice);
        assert_eq!(
            message.message(),
            "extension \"uuid-ossp\" already exists, skipping"
        );
        assert_eq!(message.code(), "42710");
        assert_eq!(message.file(), Some("extension.c"));
        assert_eq!(message.line(), Some(1656));
        assert_eq!(message.routine(), Some("CreateExtension"));

        Ok(())
    }
}
