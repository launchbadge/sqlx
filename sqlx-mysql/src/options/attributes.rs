use std::{collections::BTreeMap, str::FromStr};

/// Connection Attributes
///
/// https://dev.mysql.com/doc/x-devapi-userguide-shell-python/en/connection-attributes-xdevapi.html
/// https://dev.mysql.com/doc/connector-net/en/connector-net-8-0-connection-options.html
#[derive(Debug, Clone)]
pub(crate) enum Attributes {
    /// No client attributes are send to the server
    None,

    /// Only the default client attributes are send to the server
    ///
    /// These attributes are:
    /// * _client_name: sqlx-mysql
    /// * _client_version: The version of the sqlx crate
    ClientDefault,

    /// The default client and additional specified attributes are send to the server
    ClientDefaultAndCustom(BTreeMap<String, String>),

    /// Only the specified attributes are send to the server
    Custom(BTreeMap<String, String>),
}

/// The default is to only send the default client attributes to the server
impl Default for Attributes {
    fn default() -> Self {
        Attributes::ClientDefault
    }
}

impl FromStr for Attributes {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "false" {
            return Ok(Attributes::None);
        } else if s.is_empty() || s == "true" {
            return Ok(Attributes::ClientDefault);
        }

        // The format for custom attributes is: [key1=value1,key2=value2]
        // Remove outer [ ]
        let s = s
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or("Invalid attribute format")?;

        let mut attributes = BTreeMap::new();
        for (key, value) in s.split(',').map(|pair| pair.split_once('=')).flatten() {
            if key.is_empty() {
                return Err("Empty keys are not allowed in connection attributes");
            }

            attributes.insert(String::from(key), String::from(value));
        }

        Ok(Attributes::ClientDefaultAndCustom(attributes))
    }
}

#[test]
fn parse_attributes() {
    assert!(matches!(
    "[k1=@123,2=v2,k3=v3]".parse().unwrap(),
    Attributes::ClientDefaultAndCustom(attr) if attr == BTreeMap::from([
        (String::from("k1"), String::from("@123")),
        (String::from("2"), String::from("v2")),
        (String::from("k3"), String::from("v3")),
    ])));

    assert!(matches!("".parse().unwrap(), Attributes::ClientDefault));
    assert!(matches!("true".parse().unwrap(), Attributes::ClientDefault));
    assert!(matches!("false".parse().unwrap(), Attributes::None));
}
