use std::collections::BTreeMap;

use sqlx_core::io::Encode;

use crate::{io::MySqlBufMutExt, options::Attributes, protocol::Capabilities};

impl Encode<'_, Capabilities> for Attributes {
    fn encode_with(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // Connection attributes are not enabled or not supported
        if !capabilities.contains(Capabilities::CONNECT_ATTRS) || matches!(self, Attributes::None) {
            return;
        }

        let mut attributes_to_encode = BTreeMap::new();
        match self {
            Attributes::None => unreachable!(),
            Attributes::ClientDefault => {
                add_client_attributes(&mut attributes_to_encode);
            }

            Attributes::ClientDefaultAndCustom(custom_attributes) => {
                add_client_attributes(&mut attributes_to_encode);
                attributes_to_encode.extend(
                    custom_attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
            }

            Attributes::Custom(custom_attributes) => {
                attributes_to_encode.extend(
                    custom_attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
            }
        }

        if attributes_to_encode.is_empty() {
            return;
        }

        // Use temporary buffer to get total length of encoded key/value pairs
        let mut attribute_buffer = vec![];

        // Add key/value pairs to the buffer
        for (key, value) in attributes_to_encode {
            attribute_buffer.put_str_lenenc(key);
            attribute_buffer.put_str_lenenc(value);
        }

        // Finally add encoded connection attributes with prefixed length
        buf.put_uint_lenenc(attribute_buffer.len() as u64);
        buf.extend_from_slice(&attribute_buffer);
    }
}

/// Add default client attributes
///
/// https://dev.mysql.com/doc/refman/8.0/en/performance-schema-connection-attribute-tables.html
fn add_client_attributes(attr: &mut BTreeMap<&str, &str>) {
    attr.insert("_client_name", "sqlx-mysql");
    attr.insert("_client_version", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
#[macro_export]
macro_rules! u8_slice {
    ($($data:expr),*) => {{
        let mut r: Vec<u8> = Vec::new();
        $(match &stringify!($data)[..1] {
                "\"" => { r.extend(stringify!($data).trim_matches('"').as_bytes()) }
                _ => { r.push(stringify!($data).parse::<u8>().unwrap()) }
        })* r
    }};
}

#[test]
fn test_attributes_not_supported() {
    let capabilities = Capabilities::empty();
    let client_default = Attributes::ClientDefault;

    let mut buffer = vec![];
    client_default.encode_with(&mut buffer, capabilities);
    assert!(buffer.is_empty());
}

#[test]
fn test_attribute_encoding() {
    let capabilities = Capabilities::CONNECT_ATTRS;
    let client_default = Attributes::Custom(BTreeMap::from([
        ("attrib1".into(), "0123".into()),
        ("attrib2_empty".into(), "".into()),
        ("attrib3".into(), "456".into()),
    ]));

    let mut buffer = vec![];
    client_default.encode_with(&mut buffer, capabilities);

    #[rustfmt::skip]
    let mut encoded = u8_slice!(
        7,  "attrib1",
        4,  "0123",

        13, "attrib2_empty",
        0,

        7,  "attrib3",
        3,  "456"
    );

    // Prefix length (<251) as 1 byte
    encoded.insert(0, encoded.len() as u8);

    assert_eq!(encoded, buffer.as_slice());
}

#[test]
fn test_custom_attributes_override_client_default() {
    let capabilities = Capabilities::CONNECT_ATTRS;
    let client_default = Attributes::ClientDefaultAndCustom(BTreeMap::from([
        ("_client_version".into(), "overwrite1".into()),
        ("newattrib".into(), "123".into()),
    ]));

    let mut buffer = vec![];
    client_default.encode_with(&mut buffer, capabilities);

    #[rustfmt::skip]
    let mut encoded = u8_slice!(
        12, "_client_name",
        10, "sqlx-mysql",

        15, "_client_version",
        10, "overwrite1",

        9,  "newattrib",
        3,  "123"
    );

    // Prefix length (<251) as 1 byte
    encoded.insert(0, encoded.len() as u8);

    assert_eq!(encoded, buffer.as_slice());
}
