use byteorder::{BigEndian, ByteOrder};

// Reference
// https://www.postgresql.org/docs/devel/protocol-message-formats.html
// https://www.postgresql.org/docs/devel/protocol-message-types.html

#[derive(Debug)]
pub struct Terminate;

impl Terminate {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.push(4);
    }
}

#[derive(Debug)]
pub struct StartupMessage<'a> {
    /// One or more pairs of parameter name and value strings.
    /// A zero byte is required as a terminator after the last name/value pair.
    /// Parameters can appear in any order. user is required, others are optional.
    pub params: &'a [(&'a str, &'a str)],
}

impl<'a> StartupMessage<'a> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        with_length_prefix(buf, |buf| {
            // version: 3 = major, 0 = minor
            buf.extend_from_slice(&0x0003_0000_i32.to_be_bytes());

            for (name, value) in self.params {
                buf.extend_from_slice(name.as_bytes());
                buf.push(0);
                buf.extend_from_slice(value.as_bytes());
                buf.push(0);
            }

            // A zero byte is required as a terminator after the last name/value pair.
            buf.push(0);
        });
    }
}

// Write a variable amount of data into a buffer and then
// prefix that data with the length of what was written
fn with_length_prefix<F>(buf: &mut Vec<u8>, f: F)
where
    F: FnOnce(&mut Vec<u8>),
{
    // Reserve space for length
    let base = buf.len();
    buf.extend_from_slice(&[0; 4]);

    f(buf);

    // Write back the length
    // FIXME: Handle >= i32
    let size = (buf.len() - base) as i32;
    BigEndian::write_i32(&mut buf[base..], size);
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Serialize test more messages

    #[test]
    fn ser_startup_message() {
        let msg = StartupMessage { params: &[("user", "postgres"), ("database", "postgres")] };

        let mut buf = Vec::new();
        msg.serialize(&mut buf);

        assert_eq!(
            "00000029000300007573657200706f73746772657\
             300646174616261736500706f7374677265730000",
            hex::encode(buf)
        );
    }
}
