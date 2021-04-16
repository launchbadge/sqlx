use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU8;
use std::str::{from_utf8, FromStr};

use bytes::{Buf, Bytes};
use bytestring::ByteString;
use memchr::memchr;
use sqlx_core::io::Deserialize;

use crate::PgClientError;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PgNoticeSeverity {
    Panic,
    Fatal,
    Error,
    Warning,
    Notice,
    Debug,
    Info,
    Log,
}

impl PgNoticeSeverity {
    #[inline]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Panic | Self::Fatal | Self::Error)
    }
}

impl FromStr for PgNoticeSeverity {
    type Err = PgClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PANIC" => Self::Panic,
            "FATAL" => Self::Fatal,
            "ERROR" => Self::Error,
            "WARNING" => Self::Warning,
            "NOTICE" => Self::Notice,
            "DEBUG" => Self::Debug,
            "INFO" => Self::Info,
            "LOG" => Self::Log,

            _ => {
                return Err(PgClientError::UnknownNoticeSeverity(s.into()));
            }
        })
    }
}

impl Display for PgNoticeSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Panic => "PANIC",
            Self::Fatal => "FATAL",
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Notice => "NOTICE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Log => "LOG",
        })
    }
}

pub struct PgNotice {
    data: Bytes,
    severity: PgNoticeSeverity,
    message: ByteString,
    code: ByteString,
}

impl PgNotice {
    pub const fn severity(&self) -> PgNoticeSeverity {
        self.severity
    }

    pub fn code(&self) -> &str {
        self.code.as_ref()
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.get(b'D')
    }

    pub fn hint(&self) -> Option<&str> {
        self.get(b'H')
    }

    pub fn position(&self) -> Option<&str> {
        self.get(b'P')
    }

    pub fn internal_position(&self) -> Option<u32> {
        self.get(b'p').and_then(|s| s.parse().ok())
    }

    pub fn internal_query(&self) -> Option<&str> {
        self.get(b'q')
    }

    #[doc(alias = "where")]
    pub fn context(&self) -> Option<&str> {
        self.get(b'W')
    }

    pub fn schema_name(&self) -> Option<&str> {
        self.get(b's')
    }

    pub fn table_name(&self) -> Option<&str> {
        self.get(b't')
    }

    pub fn column_name(&self) -> Option<&str> {
        self.get(b'c')
    }

    #[doc(alias = "data_type_name")]
    pub fn type_name(&self) -> Option<&str> {
        self.get(b'd')
    }

    pub fn constraint_name(&self) -> Option<&str> {
        self.get(b'n')
    }

    pub fn file(&self) -> Option<&str> {
        self.get(b'F')
    }

    pub fn line(&self) -> Option<u32> {
        self.get(b'L').and_then(|s| s.parse().ok())
    }

    pub fn routine(&self) -> Option<&str> {
        self.get(b'R')
    }

    fn get(&self, field: u8) -> Option<&str> {
        NoticeFields(&self.data)
            .find(|(ty, value)| *ty == field)
            .and_then(|(_, value)| from_utf8(value).ok())
    }
}

impl Display for PgNotice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}] {}", self.severity(), self.code(), self.message())
    }
}

impl Debug for PgNotice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("PgNotice");

        // add the standard fields
        dbg.field("severity", &self.severity());
        dbg.field("code", &self.code());
        dbg.field("message", &self.message());

        // iterate through the remainder of the fields
        for (ty, value) in NoticeFields(&*self.data) {
            let value = if let Ok(value) = from_utf8(value) { value } else { continue };

            match ty {
                b'W' => {
                    dbg.field("context", &value);
                }

                b'D' => {
                    dbg.field("detail", &value);
                }

                b'H' => {
                    dbg.field("hint", &value);
                }

                b'P' => {
                    dbg.field("position", &value);
                }

                b'p' => {
                    dbg.field("internal_position", &value);
                }

                b'q' => {
                    dbg.field("internal_query", &value);
                }

                b's' => {
                    dbg.field("schema_name", &value);
                }

                b't' => {
                    dbg.field("table_name", &value);
                }

                b'c' => {
                    dbg.field("column_name", &value);
                }

                b'd' => {
                    dbg.field("type_name", &value);
                }

                b'n' => {
                    dbg.field("constraint_name", &value);
                }

                b'F' => {
                    dbg.field("file", &value);
                }

                b'L' => {
                    dbg.field("line", &value);
                }

                b'R' => {
                    dbg.field("routine", &value);
                }

                _ => {}
            }
        }

        dbg.finish()
    }
}

impl Deserialize<'_> for PgNotice {
    fn deserialize_with(buf: Bytes, _: ()) -> sqlx_core::Result<Self> {
        // In order to support PostgreSQL 9.5 and older we need to parse the localized S field.
        // Newer versions additionally come with the V field that is guaranteed to be in English.
        // We thus read both versions and prefer the english one if available.
        let mut fields = NoticeFields(&*buf);
        let mut severity_v: Option<Bytes> = None;
        let mut severity_s: Option<Bytes> = None;
        let mut code: Option<Bytes> = None;
        let mut message: Option<Bytes> = None;

        while let Some((ty, value)) = fields.next() {
            let value = buf.slice_ref(value);

            match ty {
                b'S' => {
                    severity_s = Some(value);
                }

                b'V' => {
                    severity_v = Some(value);
                }

                b'C' => {
                    code = Some(value);
                }

                b'M' => {
                    message = Some(value);
                }

                _ => {}
            }

            if (severity_v.is_some() || severity_s.is_some()) && message.is_some() && code.is_some()
            {
                // stop iterating through fields as soon as we found enough
                break;
            }
        }

        // default to a hopefully useful message if we can't parse the message as UTF-8
        // the message should ALWAYS be UTF-8 except for auth errors during startup
        // ref: https://github.com/launchbadge/sqlx/issues/1144#issuecomment-817043259
        let message = message
            .and_then(|message| ByteString::try_from(message).ok())
            .unwrap_or_else(|| ByteString::from_static("failed to parse error received from postgres, likely invalid authentication, confirm connection information and check database logs"));

        // code should _always_ be ASCII
        // if it is not, we default to a code of XX001 (data corrupted)
        let code = code
            .and_then(|code| ByteString::try_from(code).ok())
            .unwrap_or_else(|| ByteString::from_static("XX001"));

        // severity (v) should always be english and ASCII
        // if we are in Postgres 9.5 or older, we will only have severity (s) and if its an auth
        // error, this might not be UTF-8, in that case, we default to FATAL
        let severity = severity_v.or(severity_s);
        let severity: PgNoticeSeverity = severity
            .and_then(|code| ByteString::try_from(code).ok())
            .unwrap_or_else(|| ByteString::from_static("FATAL"))
            .parse()?;

        Ok(Self { data: buf.slice_ref(fields.0), message, severity, code })
    }
}

struct NoticeFields<'a>(&'a [u8]);

impl<'a> Iterator for NoticeFields<'a> {
    type Item = (u8, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        // the fields in the response body are sequentially stored as [tag][string],
        // ending in a final [NUL]

        // if ty is 0, we are at the end
        let ty = NonZeroU8::new(self.0.get_u8())?;

        // if there is no NUL terminator on the value, give up
        let nul = memchr(b'\0', self.0)?;

        let value = &self.0[..nul];
        self.0 = &self.0[nul + 1..];

        Some((ty.get(), value))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use sqlx_core::io::Deserialize;

    use super::{NoticeFields, PgNotice, PgNoticeSeverity};

    #[test]
    fn should_deserialize_notice() -> sqlx_core::Result<()> {
        let buf = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";
        let notice = PgNotice::deserialize(Bytes::from_static(buf))?;

        assert!(!notice.severity.is_error());
        assert_eq!(notice.severity, PgNoticeSeverity::Notice);
        assert_eq!(notice.message, "extension \"uuid-ossp\" already exists, skipping");
        assert_eq!(notice.code, "42710");

        assert_eq!(
            format!("{:?}", notice),
            "PgNotice { \
                severity: Notice, \
                code: \"42710\", \
                message: \"extension \\\"uuid-ossp\\\" already exists, skipping\", \
                file: \"extension.c\", \
                line: \"1656\", \
                routine: \"CreateExtension\" \
            }"
        );

        Ok(())
    }

    #[test]
    fn should_not_fail_deserialize_win1251_notice() -> sqlx_core::Result<()> {
        let buf = Bytes::from(vec![
            83, 194, 192, 198, 205, 206, 0, 86, 70, 65, 84, 65, 76, 0, 67, 50, 56, 80, 48, 49, 0,
            77, 239, 238, 235, 252, 231, 238, 226, 224, 242, 229, 235, 252, 32, 34, 112, 122, 105,
            120, 101, 34, 32, 237, 229, 32, 239, 240, 238, 248, 184, 235, 32, 239, 240, 238, 226,
            229, 240, 234, 243, 32, 239, 238, 228, 235, 232, 237, 237, 238, 241, 242, 232, 32, 40,
            239, 238, 32, 239, 224, 240, 238, 235, 254, 41, 0, 70, 100, 58, 92, 112, 103, 105, 110,
            115, 116, 97, 108, 108, 101, 114, 95, 49, 50, 46, 97, 117, 116, 111, 92, 112, 111, 115,
            116, 103, 114, 101, 115, 46, 119, 105, 110, 100, 111, 119, 115, 45, 120, 54, 52, 92,
            115, 114, 99, 92, 98, 97, 99, 107, 101, 110, 100, 92, 108, 105, 98, 112, 113, 92, 97,
            117, 116, 104, 46, 99, 0, 76, 51, 51, 51, 0, 82, 97, 117, 116, 104, 95, 102, 97, 105,
            108, 101, 100, 0, 0,
        ]);

        let notice = PgNotice::deserialize(buf)?;

        assert!(notice.severity.is_error());
        assert_eq!(notice.severity, PgNoticeSeverity::Fatal);
        assert_eq!(
            notice.message,
            "failed to parse error received from postgres, likely invalid authentication, confirm connection information and check database logs"
        );
        assert_eq!(notice.code, "28P01");

        Ok(())
    }

    #[test]
    fn should_parse_fields() {
        let buf = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";
        let fields: Vec<_> = NoticeFields(buf).collect();

        assert_eq!(fields.len(), 7);
        assert_eq!(fields[0], (b'S', &b"NOTICE"[..]));
        assert_eq!(fields[1], (b'V', &b"NOTICE"[..]));
        assert_eq!(fields[2], (b'C', &b"42710"[..]));
        assert_eq!(fields[3], (b'M', &b"extension \"uuid-ossp\" already exists, skipping"[..]));
        assert_eq!(fields[4], (b'F', &b"extension.c"[..]));
        assert_eq!(fields[5], (b'L', &b"1656"[..]));
        assert_eq!(fields[6], (b'R', &b"CreateExtension"[..]));
    }
}
