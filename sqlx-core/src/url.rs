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

impl TryFrom<url::Url> for Url {
    type Error = url::ParseError;

    fn try_from(value: url::Url) -> Result<Self, Self::Error> {
        Ok(Url(value))
    }
}

impl Url {
    #[allow(dead_code)]
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn host(&self) -> Option<&str> {
        match self.0.host_str()? {
            "" => None,
            host => Some(host),
        }
    }

    pub fn port(&self, default: u16) -> u16 {
        self.0.port().unwrap_or(default)
    }

    pub fn username(&self) -> Option<Cow<str>> {
        let username = self.0.username();

        if username.is_empty() {
            None
        } else {
            Some(
                percent_encoding::percent_decode_str(username)
                    .decode_utf8()
                    .expect("percent-encoded username contained non-UTF-8 bytes"),
            )
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

    /// Undo URL percent-encoding and return [authority]path[query]
    ///
    /// Mostly a hack to fix special-character handling for SQLite as its connection string is a
    /// file path and not _really_ a URL
    pub fn path_decoded(&self) -> Cow<str> {
        // omit scheme (e.g. `sqlite://`, `mysql://`)
        let url_str = &self.0.as_str()[self.0.scheme().len()..]
            .trim_start_matches(':')
            .trim_start_matches("//");

        // decode
        percent_encoding::percent_decode_str(url_str)
            .decode_utf8()
            .expect("percent-encoded path contained non-UTF-8 bytes")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn azure_connection_string_username_unencoded() {
        let connection_string =
            "postgres://username@servername:password@example.postgres.database.azure.com/db";

        let url = Url::try_from(connection_string).expect("Failed to parse URL");

        assert_eq!(
            url.username().map(|u| u.to_string()),
            Some(String::from("username@servername"))
        );
    }

    #[test]
    fn azure_connection_string_username_encoded() {
        let connection_string =
            "postgres://username%40servername:password@example.postgres.database.azure.com/db";

        let url = Url::try_from(connection_string).expect("Failed to parse URL");

        assert_eq!(
            url.username().map(|u| u.to_string()),
            Some(String::from("username@servername"))
        );
    }
}
