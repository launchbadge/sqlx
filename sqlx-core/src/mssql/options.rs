use std::str::FromStr;

use crate::error::BoxDynError;

#[derive(Debug, Clone)]
pub struct MsSqlConnectOptions {}

impl FromStr for MsSqlConnectOptions {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}
