use crate::error::Error;
use crate::sqlite::SqliteConnectOptions;
use percent_encoding::percent_decode_str;
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

// https://www.sqlite.org/uri.html

impl FromStr for SqliteConnectOptions {
    type Err = Error;

    fn from_str(mut uri: &str) -> Result<Self, Self::Err> {
        let mut options = Self::new();

        // remove scheme from the URI
        uri = uri
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        let mut database_and_params = uri.splitn(2, '?');

        let database = database_and_params.next().unwrap_or_default();

        if database == ":memory:" {
            options.in_memory = true;
        } else {
            // % decode to allow for `?` or `#` in the filename
            options.filename = Cow::Owned(
                Path::new(
                    &*percent_decode_str(database)
                        .decode_utf8()
                        .map_err(Error::config)?,
                )
                .to_path_buf(),
            );
        }

        if let Some(params) = database_and_params.next() {
            for (key, value) in url::form_urlencoded::parse(params.as_bytes()) {
                match &*key {
                    // The mode query parameter determines if the new database is opened read-only,
                    // read-write, read-write and created if it does not exist, or that the
                    // database is a pure in-memory database that never interacts with disk,
                    // respectively.
                    "mode" => {
                        match &*value {
                            "ro" => {
                                options.read_only = true;
                            }

                            // default
                            "rw" => {}

                            "rwc" => {
                                options.create_if_missing = true;
                            }

                            "memory" => {
                                options.in_memory = true;
                            }

                            _ => {
                                return Err(Error::Configuration(
                                    format!("unknown value {:?} for `mode`", value).into(),
                                ));
                            }
                        }
                    }

                    _ => {
                        return Err(Error::Configuration(
                            format!(
                                "unknown query parameter `{}` while parsing connection URI",
                                key
                            )
                            .into(),
                        ));
                    }
                }
            }
        }

        Ok(options)
    }
}

#[test]
fn test_parse_in_memory() -> Result<(), Error> {
    let options: SqliteConnectOptions = "sqlite::memory:".parse()?;
    assert!(options.in_memory);

    let options: SqliteConnectOptions = "sqlite://?mode=memory".parse()?;
    assert!(options.in_memory);

    let options: SqliteConnectOptions = "sqlite://:memory:".parse()?;
    assert!(options.in_memory);

    Ok(())
}

#[test]
fn test_parse_read_only() -> Result<(), Error> {
    let options: SqliteConnectOptions = "sqlite://a.db?mode=ro".parse()?;
    assert!(options.read_only);
    assert_eq!(&*options.filename.to_string_lossy(), "a.db");

    Ok(())
}
