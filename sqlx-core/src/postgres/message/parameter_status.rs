use bytes::Bytes;

use crate::error::Error;
use crate::io::{BufExt, Decode};

#[derive(Debug)]
pub struct ParameterStatus {
    pub(crate) key: String,
    pub(crate) val: String,
}
impl Decode<'_> for ParameterStatus {
    #[inline]
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self, Error> {
        let key = buf.get_str_nul()?.to_owned();
        let val = buf.get_str_nul()?.to_owned();

        Ok(Self { key, val })
    }
}

#[test]
fn test_decode_parameter_status_response() {
    const PARAMETER_STATUS_RESPONSE: &[u8] = b"crdb_version\0CockroachDB CCL v21.1.0 (x86_64-unknown-linux-gnu, built 2021/05/17 13:49:40, go1.15.11)\0";

    let message = ParameterStatus::decode(Bytes::from(PARAMETER_STATUS_RESPONSE)).unwrap();

    assert_eq!(message.key, "crdb_version");
    assert_eq!(
        message.val,
        "CockroachDB CCL v21.1.0 (x86_64-unknown-linux-gnu, built 2021/05/17 13:49:40, go1.15.11)"
    );
}
