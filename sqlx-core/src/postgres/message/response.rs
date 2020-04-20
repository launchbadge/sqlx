use std::str::from_utf8;

use bytes::Bytes;
use memchr::memchr;

use crate::error::Error;
use crate::io::Decode;

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub struct Response {
    storage: Bytes,
    severity: PgSeverity,
    message: (u16, u16),
    code: (u16, u16),
}

impl Response {
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

impl Response {
    #[inline]
    fn fields(&self) -> Fields<'_> {
        Fields {
            storage: &self.storage,
            offset: 0,
        }
    }

    #[inline]
    fn get_cached_str(&self, cache: (u16, u16)) -> &str {
        // unwrap: this cannot fail at this stage
        from_utf8(&self.storage[cache.0 as usize..cache.1 as usize]).unwrap()
    }
}

impl Decode for Response {
    fn decode(buf: Bytes) -> Result<Self, Error> {
        let mut severity = PgSeverity::Log;
        let mut message = (0, 0);
        let mut code = (0, 0);

        // we cache the three always present fields
        // this enables to keep the access time down for the fields most likely accessed

        let fields = Fields {
            storage: &buf,
            offset: 0,
        };

        for (field, v) in fields {
            if message.0 != 0 && code.0 != 0 {
                // stop iterating when we have the 3 fields we were looking for
                // we assume V (severity) was the first field as it should be
                break;
            }

            match field {
                b'V' => {
                    // unwrap: impossible to fail at this point
                    severity = match from_utf8(&buf[v.0 as usize..v.1 as usize]).unwrap() {
                        "PANIC" => PgSeverity::Panic,
                        "FATAL" => PgSeverity::Fatal,
                        "ERROR" => PgSeverity::Error,
                        "WARNING" => PgSeverity::Warning,
                        "NOTICE" => PgSeverity::Notice,
                        "DEBUG" => PgSeverity::Debug,
                        "INFO" => PgSeverity::Info,
                        "LOG" => PgSeverity::Log,

                        severity => {
                            return Err(err_protocol!("unknown severity: {:?}", severity));
                        }
                    };
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
            severity,
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

#[test]
fn test_decode_error_response() {
    const DATA: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    let m = Response::decode(Bytes::from_static(DATA)).unwrap();

    assert_eq!(
        m.message(),
        "extension \"uuid-ossp\" already exists, skipping"
    );

    assert!(matches!(m.severity(), PgSeverity::Notice));
    assert_eq!(m.code(), "42710");
}

#[cfg(all(test, not(debug_assertions)))]
#[bench]
fn bench_error_response_get_message(b: &mut test::Bencher) {
    const DATA: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    let res = Response::decode(test::black_box(Bytes::from_static(DATA))).unwrap();

    b.iter(|| {
        let _ = test::black_box(&res).message();
    });
}

#[cfg(all(test, not(debug_assertions)))]
#[bench]
fn bench_decode_error_response(b: &mut test::Bencher) {
    const DATA: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    b.iter(|| {
        let _ = Response::decode(test::black_box(Bytes::from_static(DATA)));
    });
}
