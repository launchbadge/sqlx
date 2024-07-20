use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    str::FromStr,
};

/// Connection Attributes
///
/// The default is to not send any client attributes to the server
///
/// https://dev.mysql.com/doc/x-devapi-userguide-shell-python/en/connection-attributes-xdevapi.html
/// https://dev.mysql.com/doc/connector-net/en/connector-net-8-0-connection-options.html
#[derive(Debug, Default, Clone)]
pub(crate) struct Attributes(BTreeMap<String, String>);

impl Attributes {
    /// Add default client attributes
    ///
    /// https://dev.mysql.com/doc/refman/8.0/en/performance-schema-connection-attribute-tables.html
    pub(crate) fn add_default_client_attributes(&mut self) {
        self.0
            .insert("_client_name".to_string(), "sqlx-mysql".to_string());
        self.0.insert(
            "_client_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
    }
}

impl Deref for Attributes {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Implement parsing connection attributes from the connection string
impl FromStr for Attributes {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut attributes = Self::default();

        if s == "false" || s.is_empty() {
            return Ok(attributes);
        } else if s == "true" {
            attributes.add_default_client_attributes();
            return Ok(attributes);
        }

        // The format for custom attributes is: [key1=value1,key2=value2]
        // Remove outer [ ]
        let s = s
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or("Invalid attribute format")?;

        for (key, value) in s.split(',').map(|pair| pair.split_once('=')).flatten() {
            if key.is_empty() {
                return Err("Empty keys are not allowed in connection attributes");
            }

            attributes.insert(String::from(key), String::from(value));
        }

        Ok(attributes)
    }
}

#[test]
fn parse_attributes() {
    assert!(matches!(
    "[k1=@123,2=v2,k3=v3]".parse().unwrap(),
    Attributes(attr) if attr == BTreeMap::from([
        (String::from("k1"), String::from("@123")),
        (String::from("2"), String::from("v2")),
        (String::from("k3"), String::from("v3")),
    ])));

    assert!("".parse::<Attributes>().unwrap().is_empty());
    assert!("false".parse::<Attributes>().unwrap().is_empty());

    let default_attrib = "true".parse::<Attributes>().unwrap();
    assert_eq!(default_attrib.get("_client_name").unwrap(), "sqlx-mysql");
    assert_eq!(default_attrib.get("_client_version").unwrap(), env!("CARGO_PKG_VERSION"));
}
