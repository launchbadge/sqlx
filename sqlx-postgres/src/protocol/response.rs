use std::str::from_utf8;

use bytes::Bytes;
use memchr::memchr;
use sqlx_core::io::Deserialize;
use sqlx_core::Error;
use sqlx_core::Result;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PgSeverity {
    Panic,
    Fatal,
    Error,
    Warning,
    Notice,
    Debug,
    Info,
    Log,
}

impl PgSeverity {
    #[inline]
    pub fn is_error(self) -> bool {
        matches!(self, Self::Panic | Self::Fatal | Self::Error)
    }
}

impl std::convert::TryFrom<&str> for PgSeverity {
    type Error = Error;

    fn try_from(s: &str) -> Result<PgSeverity> {
        let result = match s {
            "PANIC" => PgSeverity::Panic,
            "FATAL" => PgSeverity::Fatal,
            "ERROR" => PgSeverity::Error,
            "WARNING" => PgSeverity::Warning,
            "NOTICE" => PgSeverity::Notice,
            "DEBUG" => PgSeverity::Debug,
            "INFO" => PgSeverity::Info,
            "LOG" => PgSeverity::Log,

            severity => {
                return Err(Error::configuration_msg(format!("unknown severity: {:?}", severity)));
            }
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct Notice {
    storage: Bytes,
    severity: PgSeverity,
    message: (u16, u16),
    code: (u16, u16),
}

impl Notice {
    #[inline]
    pub fn severity(&self) -> PgSeverity {
        self.severity
    }

    #[inline]
    pub fn code(&self) -> &str {
        self.get_cached_str(self.code)
    }

    #[inline]
    pub fn message(&self) -> &str {
        self.get_cached_str(self.message)
    }

    // Field descriptions available here:
    //  https://www.postgresql.org/docs/current/protocol-error-fields.html

    #[inline]
    pub fn get(&self, ty: u8) -> Option<&str> {
        self.get_raw(ty).and_then(|v| from_utf8(v).ok())
    }

    pub fn get_raw(&self, ty: u8) -> Option<&[u8]> {
        self.fields()
            .filter(|(field, _)| *field == ty)
            .map(|(_, (start, end))| &self.storage[start as usize..end as usize])
            .next()
    }
}

impl Notice {
    #[inline]
    fn fields(&self) -> Fields<'_> {
        Fields { storage: &self.storage, offset: 0 }
    }

    #[inline]
    fn get_cached_str(&self, cache: (u16, u16)) -> &str {
        // unwrap: this cannot fail at this stage
        from_utf8(&self.storage[cache.0 as usize..cache.1 as usize]).unwrap()
    }
}

impl Deserialize<'_, ()> for Notice {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        // In order to support PostgreSQL 9.5 and older we need to parse the localized S field.
        // Newer versions additionally come with the V field that is guaranteed to be in English.
        // We thus read both versions and prefer the unlocalized one if available.
        const DEFAULT_SEVERITY: PgSeverity = PgSeverity::Log;
        let mut severity_v = None;
        let mut severity_s = None;
        let mut message = (0, 0);
        let mut code = (0, 0);

        // we cache the three always present fields
        // this enables to keep the access time down for the fields most likely accessed

        let fields = Fields { storage: &buf, offset: 0 };

        for (field, v) in fields {
            if message.0 != 0 && code.0 != 0 {
                // stop iterating when we have the 3 fields we were looking for
                // we assume V (severity) was the first field as it should be
                break;
            }

            use std::convert::TryInto;
            match field {
                b'S' => {
                    // Discard potential errors, because the message might be localized
                    severity_s =
                        from_utf8(&buf[v.0 as usize..v.1 as usize]).unwrap().try_into().ok();
                }

                b'V' => {
                    // Propagate errors here, because V is not localized and thus we are missing a possible
                    // variant.
                    severity_v =
                        Some(from_utf8(&buf[v.0 as usize..v.1 as usize]).unwrap().try_into()?);
                }

                b'M' => {
                    message = v;
                }

                b'C' => {
                    code = v;
                }

                _ => {}
            }
        }

        Ok(Self {
            severity: severity_v.or(severity_s).unwrap_or(DEFAULT_SEVERITY),
            message,
            code,
            storage: buf,
        })
    }
}

/// An iterator over each field in the Error (or Notice) response.
struct Fields<'a> {
    storage: &'a [u8],
    offset: u16,
}

impl<'a> Iterator for Fields<'a> {
    type Item = (u8, (u16, u16));

    fn next(&mut self) -> Option<Self::Item> {
        // The fields in the response body are sequentially stored as [tag][string],
        // ending in a final, additional [nul]

        let ty = self.storage[self.offset as usize];

        if ty == 0 {
            return None;
        }

        let nul = memchr(b'\0', &self.storage[(self.offset + 1) as usize..])? as u16;
        let offset = self.offset;

        self.offset += nul + 2;

        Some((ty, (offset + 1, offset + nul + 1)))
    }
}
