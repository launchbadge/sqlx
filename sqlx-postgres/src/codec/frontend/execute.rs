use crate::io::{put_length_prefixed, put_portal_name};
use sqlx_core::{error::Error, io::Encode};

pub(crate) struct Execute {
    /// The id of the portal to execute (`None` selects the unnamed portal).
    pub(crate) portal: Option<u32>,

    /// Maximum number of rows to return, if portal contains a query
    /// that returns rows (ignored otherwise). Zero denotes “no limit”.
    pub(crate) limit: i32,
}

impl Encode<'_> for Execute {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        buf.push(b'E');

        put_length_prefixed(buf, true, |buf| {
            put_portal_name(buf, self.portal);
            buf.extend(&self.limit.to_be_bytes());

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_named() {
        const EXPECTED: &[u8] = b"E\0\0\0\x11sqlx_p_5\0\0\0\0\x02";

        let mut buf = Vec::new();

        let m = Execute {
            portal: Some(5),
            limit: 2,
        };

        m.encode(&mut buf);

        assert_eq!(buf, EXPECTED);
    }
}
