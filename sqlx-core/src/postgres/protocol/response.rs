use crate::io::Buf;
use crate::postgres::protocol::Decode;
use std::str::{self, FromStr};

#[derive(Debug, Copy, Clone)]
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
}

impl FromStr for Severity {
    type Err = crate::Error;

    fn from_str(s: &str) -> crate::Result<Self> {
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
                return Err(protocol_err!("unexpected response severity: {}", s).into());
            }
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub severity: Severity,
    pub code: Box<str>,
    pub message: Box<str>,
    pub detail: Option<Box<str>>,
    pub hint: Option<Box<str>>,
    pub position: Option<usize>,
    pub internal_position: Option<usize>,
    pub internal_query: Option<Box<str>>,
    pub where_: Option<Box<str>>,
    pub schema: Option<Box<str>>,
    pub table: Option<Box<str>>,
    pub column: Option<Box<str>>,
    pub data_type: Option<Box<str>>,
    pub constraint: Option<Box<str>>,
    pub file: Option<Box<str>>,
    pub line: Option<usize>,
    pub routine: Option<Box<str>>,
}

impl Decode for Response {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let mut code = None::<Box<str>>;
        let mut message = None::<Box<str>>;
        let mut severity = None::<Box<str>>;
        let mut severity_non_local = None::<Severity>;
        let mut detail = None::<Box<str>>;
        let mut hint = None::<Box<str>>;
        let mut position = None::<usize>;
        let mut internal_position = None::<usize>;
        let mut internal_query = None::<Box<str>>;
        let mut where_ = None::<Box<str>>;
        let mut schema = None::<Box<str>>;
        let mut table = None::<Box<str>>;
        let mut column = None::<Box<str>>;
        let mut data_type = None::<Box<str>>;
        let mut constraint = None::<Box<str>>;
        let mut file = None::<Box<str>>;
        let mut line = None::<usize>;
        let mut routine = None::<Box<str>>;

        loop {
            let field_type = buf.get_u8()?;

            if field_type == 0 {
                break;
            }

            let field_value = buf.get_str_nul()?;

            match field_type {
                b'S' => {
                    severity = Some(field_value.into());
                }

                b'V' => {
                    severity_non_local = Some(field_value.parse()?);
                }

                b'C' => {
                    code = Some(field_value.into());
                }

                b'M' => {
                    message = Some(field_value.into());
                }

                b'D' => {
                    detail = Some(field_value.into());
                }

                b'H' => {
                    hint = Some(field_value.into());
                }

                b'P' => {
                    position = Some(
                        field_value
                            .parse()
                            .or(Err(protocol_err!("expected int, got: {}", field_value)))?,
                    );
                }

                b'p' => {
                    internal_position = Some(
                        field_value
                            .parse()
                            .or(Err(protocol_err!("expected int, got: {}", field_value)))?,
                    );
                }

                b'q' => {
                    internal_query = Some(field_value.into());
                }

                b'w' => {
                    where_ = Some(field_value.into());
                }

                b's' => {
                    schema = Some(field_value.into());
                }

                b't' => {
                    table = Some(field_value.into());
                }

                b'c' => {
                    column = Some(field_value.into());
                }

                b'd' => {
                    data_type = Some(field_value.into());
                }

                b'n' => {
                    constraint = Some(field_value.into());
                }

                b'F' => {
                    file = Some(field_value.into());
                }

                b'L' => {
                    line = Some(
                        field_value
                            .parse()
                            .or(Err(protocol_err!("expected int, got: {}", field_value)))?,
                    );
                }

                b'R' => {
                    routine = Some(field_value.into());
                }

                _ => {
                    // TODO: Should we return these somehow, like in a map?
                    return Err(protocol_err!(
                        "received unknown field in Response: {}",
                        field_type
                    )
                    .into());
                }
            }
        }

        let severity = severity_non_local
            .or_else(move || severity?.as_ref().parse().ok())
            .ok_or(protocol_err!(
                "did not receieve field `severity` for Response"
            ))?;

        let code = code.ok_or(protocol_err!("did not receieve field `code` for Response",))?;
        let message = message.ok_or(protocol_err!(
            "did not receieve field `message` for Response"
        ))?;

        Ok(Self {
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
    use matches::assert_matches;

    const RESPONSE: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, \
          skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    #[test]
    fn it_decodes_response() {
        let message = Response::decode(RESPONSE).unwrap();

        assert_matches!(message.severity, Severity::Notice);
        assert_eq!(&*message.code, "42710");
        assert_eq!(&*message.file.unwrap(), "extension.c");
        assert_eq!(message.line, Some(1656));
        assert_eq!(&*message.routine.unwrap(), "CreateExtension");
        assert_eq!(
            &*message.message,
            "extension \"uuid-ossp\" already exists, skipping"
        );
    }
}
