use crate::aurora::options::AuroraConnectOptions;
use crate::error::Error;
use std::str::FromStr;
use url::Url;

impl FromStr for AuroraConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;

        let mut options = Self::default();

        for (key, value) in url.query_pairs().into_iter() {
            match &*key {
                "statement-cache-capacity" => {
                    options =
                        options.statement_cache_capacity(value.parse().map_err(Error::config)?);
                }
                "db-type" => {
                    options = options.db_type(value.parse()?);
                }
                "region" => {
                    options = options.region(&*value);
                }
                "resource-arn" => {
                    options = options.resource_arn(&*value);
                }
                "secret-arn" => {
                    options = options.secret_arn(&*value);
                }
                _ => log::warn!("ignoring unrecognized connect parameter: {}={}", key, value),
            }
        }

        Ok(options)
    }
}
