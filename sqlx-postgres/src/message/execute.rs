use crate::io::Encode;
use crate::io::PgBufMutExt;
use crate::types::Oid;

pub struct Execute {
    /// The id of the portal to execute (`None` selects the unnamed portal).
    pub portal: Option<Oid>,

    /// Maximum number of rows to return, if portal contains a query
    /// that returns rows (ignored otherwise). Zero denotes “no limit”.
    pub limit: u32,
}

impl Encode<'_> for Execute {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        buf.reserve(20);
        buf.push(b'E');

        buf.put_length_prefixed(|buf| {
            buf.put_portal_name(self.portal);
            buf.extend(&self.limit.to_be_bytes());
        });
    }
}

#[test]
fn test_encode_execute() {
    const EXPECTED: &[u8] = b"E\0\0\0\x11sqlx_p_5\0\0\0\0\x02";

    let mut buf = Vec::new();
    let m = Execute {
        portal: Some(Oid(5)),
        limit: 2,
    };

    m.encode(&mut buf);

    assert_eq!(buf, EXPECTED);
}
