use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use percent_encoding::{percent_decode_str, percent_encode, AsciiSet};
use url::Url;

use crate::error::Error;
use crate::SqliteConnectOptions;

use super::SqliteJournalMode;

// https://www.sqlite.org/uri.html

static IN_MEMORY_DB_SEQ: AtomicUsize = AtomicUsize::new(0);

impl SqliteConnectOptions {
    pub(crate) fn from_db_and_params(database: &str, params: Option<&str>) -> Result<Self, Error> {
        let mut options = Self::default();

        if database == ":memory:" {
            options.in_memory = true;
            options.shared_cache = true;
            let seqno = IN_MEMORY_DB_SEQ.fetch_add(1, Ordering::Relaxed);
            options.filename = Cow::Owned(PathBuf::from(format!("file:sqlx-in-memory-{seqno}")));
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

        if let Some(params) = params {
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
                                options.shared_cache = true;
                            }

                            _ => {
                                return Err(Error::Configuration(
                                    format!("unknown value {value:?} for `mode`").into(),
                                ));
                            }
                        }
                    }

                    // The cache query parameter specifies the cache behaviour across multiple
                    // connections to the same database within the process. A shared cache is
                    // essential for persisting data across connections to an in-memory database.
                    "cache" => match &*value {
                        "private" => {
                            options.shared_cache = false;
                        }

                        "shared" => {
                            options.shared_cache = true;
                        }

                        _ => {
                            return Err(Error::Configuration(
                                format!("unknown value {value:?} for `cache`").into(),
                            ));
                        }
                    },

                    "immutable" => match &*value {
                        "true" | "1" => {
                            options.immutable = true;
                        }
                        "false" | "0" => {
                            options.immutable = false;
                        }
                        _ => {
                            return Err(Error::Configuration(
                                format!("unknown value {value:?} for `immutable`").into(),
                            ));
                        }
                    },

                    "vfs" => options.vfs = Some(Cow::Owned(value.into_owned())),

                    // The journal_mode query parameter specifies the journal mode to use for the database.
                    // The default is DELETE, but WAL is recommended for most use cases.
                    // See https://www.sqlite.org/pragma.html#pragma_journal_mode
                    // as journal_mode is not a standard query parameter, we prefix it with `_`
                    "_journal_mode" => {
                        if SqliteJournalMode::from_str(&value).is_err() {
                            return Err(Error::Configuration(
                                format!("unknown value {value:?} for `journal_mode`").into(),
                            ));
                        }
                        options
                            .pragmas
                            .insert("journal_mode".into(), Some(value.into_owned().into()));
                    }

                    // The busy_timeout query parameter specifies the timeout to use when the database is busy.
                    // the default is 5 seconds, but this can be changed to a shorter or longer duration.
                    // See https://www.sqlite.org/pragma.html#pragma_busy_timeout
                    // as busy_timeout is not a standard query parameter, we prefix it with `_`
                    "_busy_timeout" => {
                        if let Some(timeout) = parse_duration(&value) {
                            options.busy_timeout = timeout;
                        }
                    }

                    _ => {
                        return Err(Error::Configuration(
                            format!("unknown query parameter `{key}` while parsing connection URL")
                                .into(),
                        ));
                    }
                }
            }
        }

        Ok(options)
    }

    pub(crate) fn build_url(&self) -> Url {
        // https://url.spec.whatwg.org/#path-percent-encode-set
        static PATH_ENCODE_SET: AsciiSet = percent_encoding::CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'<')
            .add(b'>')
            .add(b'?')
            .add(b'`')
            .add(b'{')
            .add(b'}');

        let filename_encoded = percent_encode(
            self.filename.as_os_str().as_encoded_bytes(),
            &PATH_ENCODE_SET,
        );

        let mut url = Url::parse(&format!("sqlite://{filename_encoded}"))
            .expect("BUG: generated un-parseable URL");

        let mode = match (self.in_memory, self.create_if_missing, self.read_only) {
            (true, _, _) => "memory",
            (false, true, _) => "rwc",
            (false, false, true) => "ro",
            (false, false, false) => "rw",
        };
        url.query_pairs_mut().append_pair("mode", mode);

        let cache = match self.shared_cache {
            true => "shared",
            false => "private",
        };
        url.query_pairs_mut().append_pair("cache", cache);
        if self.immutable {
            url.query_pairs_mut().append_pair("immutable", "true");
        }

        if let Some(vfs) = &self.vfs {
            url.query_pairs_mut().append_pair("vfs", vfs);
        }

        if let Some(Some(journal_mode)) = self.pragmas.get("journal_mode") {
            url.query_pairs_mut()
                .append_pair("_journal_mode", journal_mode);
        }

        if !self.busy_timeout.is_zero() {
            url.query_pairs_mut()
                .append_pair("_busy_timeout", &format_duration(self.busy_timeout));
        }

        url
    }
}

impl FromStr for SqliteConnectOptions {
    type Err = Error;

    fn from_str(mut url: &str) -> Result<Self, Self::Err> {
        // remove scheme from the URL
        url = url
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        let mut database_and_params = url.splitn(2, '?');

        let database = database_and_params.next().unwrap_or_default();
        let params = database_and_params.next();

        Self::from_db_and_params(database, params)
    }
}

type TimeUnitParser = (&'static str, fn(u64) -> Duration);

// This function is used to parse the busy timeout from the URL query parameters.
// as busy timeout should be short, we only support milliseconds and seconds
fn parse_duration(s: &str) -> Option<Duration> {
    static UNITS: [TimeUnitParser; 2] = [("ms", Duration::from_millis), ("s", Duration::from_secs)];
    for (suffix, func) in UNITS.iter() {
        let Some(suffix) = s.strip_suffix(suffix) else {
            continue;
        };
        let value = suffix.parse::<u64>().ok()?;
        return Some(func(value));
    }
    None
}

fn format_duration(duration: Duration) -> String {
    if duration.subsec_millis() == 0 {
        format!("{}s", duration.as_secs())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

#[test]
fn test_parse_in_memory() -> Result<(), Error> {
    let options: SqliteConnectOptions = "sqlite::memory:".parse()?;
    assert!(options.in_memory);
    assert!(options.shared_cache);

    let options: SqliteConnectOptions = "sqlite://?mode=memory".parse()?;
    assert!(options.in_memory);
    assert!(options.shared_cache);

    let options: SqliteConnectOptions = "sqlite://:memory:".parse()?;
    assert!(options.in_memory);
    assert!(options.shared_cache);

    let options: SqliteConnectOptions = "sqlite://?mode=memory&cache=private".parse()?;
    assert!(options.in_memory);
    assert!(!options.shared_cache);

    Ok(())
}

#[test]
fn test_parse_read_only() -> Result<(), Error> {
    let options: SqliteConnectOptions = "sqlite://a.db?mode=ro".parse()?;
    assert!(options.read_only);
    assert_eq!(&*options.filename.to_string_lossy(), "a.db");

    Ok(())
}

#[test]
fn test_parse_shared_in_memory() -> Result<(), Error> {
    let options: SqliteConnectOptions = "sqlite://a.db?cache=shared".parse()?;
    assert!(options.shared_cache);
    assert_eq!(&*options.filename.to_string_lossy(), "a.db");

    Ok(())
}

#[test]
fn it_returns_the_parsed_url() -> Result<(), Error> {
    let url = "sqlite://test.db?mode=rw&cache=shared&_busy_timeout=5s";
    let options: SqliteConnectOptions = url.parse()?;

    let expected_url = Url::parse(url).unwrap();
    assert_eq!(options.build_url(), expected_url);

    Ok(())
}

#[test]
fn it_parse_journal_mode() -> Result<(), Error> {
    let url = "sqlite://test.db?_journal_mode=WAL";
    let options: SqliteConnectOptions = url.parse()?;

    let val = options.pragmas.get("journal_mode").cloned().flatten();
    assert_eq!(val, Some(Cow::Owned("WAL".to_string())));

    let format_url = options.build_url();
    assert_eq!(
        format_url.as_str(),
        "sqlite://test.db?mode=rw&cache=private&_journal_mode=WAL&_busy_timeout=5s"
    );
    Ok(())
}

#[test]
fn it_should_return_error_for_invalid_journal_mode() -> Result<(), Error> {
    let url = "sqlite://test.db?_journal_mode=invalid";
    let options: Result<SqliteConnectOptions, Error> = url.parse();

    assert!(options.is_err());
    assert_eq!(
        options.unwrap_err().to_string(),
        "error with configuration: unknown value \"invalid\" for `journal_mode`"
    );
    Ok(())
}

#[test]
fn it_should_parse_busy_timeout() -> Result<(), Error> {
    let url = "sqlite://test.db?_busy_timeout=1000ms";
    let options: SqliteConnectOptions = url.parse()?;

    assert_eq!(options.busy_timeout, Duration::from_millis(1000));

    let format_url = options.build_url();
    assert_eq!(
        format_url.as_str(),
        "sqlite://test.db?mode=rw&cache=private&_busy_timeout=1s"
    );
    Ok(())
}
