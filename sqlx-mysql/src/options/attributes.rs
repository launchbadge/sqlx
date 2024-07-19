use std::{collections::BTreeMap, str::FromStr};

/// Connection Attributes
///
/// https://dev.mysql.com/doc/x-devapi-userguide-shell-python/en/connection-attributes-xdevapi.html
/// https://dev.mysql.com/doc/connector-net/en/connector-net-8-0-connection-options.html
#[derive(Debug, Clone)]
pub(crate) enum Attributes {
    /// No client attributes are send to the server
    None,

    /// The defined attributes are send to the server
    Some(BTreeMap<String, String>),
}

impl Attributes {
    /// Add default client attributes
    ///
    /// https://dev.mysql.com/doc/refman/8.0/en/performance-schema-connection-attribute-tables.html
    pub(crate) fn add_default_client_attributes(&mut self) {
        let attr = match self {
            Attributes::None => {
                *self = Attributes::Some(BTreeMap::new());

                let Attributes::Some(ref mut new_attributes) = self else {
                    unreachable!()
                };
                new_attributes
            }
            Attributes::Some(attr) => attr,
        };

        attr.insert("_client_name".to_string(), "sqlx-mysql".to_string());
        attr.insert(
            "_client_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
    }
}

/// The default is to not send any client attributes to the server
impl Default for Attributes {
    fn default() -> Self {
        Attributes::None
    }
}

/// Implement parsing connection attributes from the connection string
impl FromStr for Attributes {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "false" || s.is_empty() {
            return Ok(Attributes::None);
        } else if s == "true" {
            let mut attributes = Attributes::None;
            attributes.add_default_client_attributes();

            return Ok(attributes);
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

        Ok(Attributes::Some(attributes))
    }
}

#[test]
fn parse_attributes() {
    assert!(matches!(
    "[k1=@123,2=v2,k3=v3]".parse().unwrap(),
    Attributes::Some(attr) if attr == BTreeMap::from([
        (String::from("k1"), String::from("@123")),
        (String::from("2"), String::from("v2")),
        (String::from("k3"), String::from("v3")),
    ])));

    assert!(matches!("".parse().unwrap(), Attributes::None));
    assert!(matches!("true".parse().unwrap(), Attributes::Some(_)));
    assert!(matches!("false".parse().unwrap(), Attributes::None));
}
