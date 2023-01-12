use crate::error::Error;
use std::str::FromStr;

/// Options for controlling the level of protection provided for PostgreSQL high availability.
///
/// It is used by the [`target_session_attrs`](super::PgConnectOptions::target_session_attrs) method.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum TargetSessionAttrs {
    /// No special properties are required.
    #[default]
    Any,
    /// The session must allow writes.
    ReadWrite,
}

impl FromStr for TargetSessionAttrs {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &*s.to_ascii_lowercase() {
            "any" => TargetSessionAttrs::Any,
            "read-write" => TargetSessionAttrs::ReadWrite,

            _ => {
                return Err(Error::Configuration(
                    format!("unknown value {:?} for `target_session_attrs`", s).into(),
                ));
            }
        })
    }
}
