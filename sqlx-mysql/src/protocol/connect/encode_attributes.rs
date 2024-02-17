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
