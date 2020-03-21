use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct Url(url::Url);

impl TryFrom<String> for Url {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<'s> TryFrom<&'s str> for Url {
    type Error = url::ParseError;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        Ok(Url(value.parse()?))
    }
}

impl<'s> TryFrom<&'s String> for Url {
    type Error = url::ParseError;

    fn try_from(value: &'s String) -> Result<Self, Self::Error> {
        (value.as_str()).try_into()
    }
}

impl Url {
    #[allow(dead_code)]
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn host(&self) -> &str {
        let host = self.0.host_str();

        match host {
            Some(host) if !host.is_empty() => host,

            _ => "localhost",
        }
    }

    pub fn port(&self, default: u16) -> u16 {
        self.0.port().unwrap_or(default)
    }

    pub fn username(&self) -> Option<&str> {
        let username = self.0.username();

        if username.is_empty() {
            None
        } else {
            Some(username)
        }
    }

    pub fn password(&self) -> Option<Cow<str>> {
        match self.0.password() {
            Some(s) => {
                let decoded = percent_encoding::percent_decode_str(s);

                // FIXME: Handle error
                Some(
                    decoded
                        .decode_utf8()
                        .expect("percent-encoded password contained non-UTF-8 bytes"),
                )
            }
            None => None,
        }
    }

    pub fn database(&self) -> Option<&str> {
        let database = self.0.path().trim_start_matches('/');

        if database.is_empty() {
            None
        } else {
            Some(database)
        }
    }

    pub fn param(&self, key: &str) -> Option<Cow<str>> {
        self.0
            .query_pairs()
            .find_map(|(key_, val)| if key == key_ { Some(val) } else { None })
    }
}
