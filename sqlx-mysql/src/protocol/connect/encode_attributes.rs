use sqlx_core::io::Encode;

use crate::{io::MySqlBufMutExt, options::Attributes, protocol::Capabilities};

/// Encode the connection attributes to the wire format
impl Encode<'_, Capabilities> for Attributes {
    fn encode_with(&self, buf: &mut Vec<u8>, capabilities: Capabilities) {
        // Connection attributes are not enabled or not supported
        if !capabilities.contains(Capabilities::CONNECT_ATTRS) || matches!(self, Attributes::None) {
            return;
        }

        let Attributes::Some(attributes) = self else {
            return;
        };

        if attributes.is_empty() {
            return;
        }

        // Use temporary buffer to get total length of encoded key/value pairs
        let mut attribute_buffer = vec![];

        // Add key/value pairs to the buffer
        for (key, value) in attributes {
            attribute_buffer.put_str_lenenc(key);
            attribute_buffer.put_str_lenenc(value);
        }

        // Finally add encoded connection attributes with prefixed length
        buf.put_uint_lenenc(attribute_buffer.len() as u64);
        buf.extend_from_slice(&attribute_buffer);
    }
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
    let client_default = Attributes::Some(std::collections::BTreeMap::from([(
        "attrib1".into(),
        "0123".into(),
    )]));

    let mut buffer = vec![];
    client_default.encode_with(&mut buffer, capabilities);
    assert!(buffer.is_empty());
}

#[test]
fn test_attribute_encoding() {
    let capabilities = Capabilities::CONNECT_ATTRS;
    let client_default = Attributes::Some(std::collections::BTreeMap::from([
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
