use byteorder::{BigEndian, ByteOrder};

// Reference
// https://www.postgresql.org/docs/devel/protocol-message-formats.html
// https://www.postgresql.org/docs/devel/protocol-message-types.html

pub trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

#[derive(Debug)]
pub struct Terminate;

impl Serialize for Terminate {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(b'X');
        buf.push(4);
    }
}

#[derive(Debug)]
pub struct StartupMessage<'a> {
    pub params: &'a [(&'a str, Option<&'a str>)],
}

impl<'a> Serialize for StartupMessage<'a> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        with_length_prefix(buf, |buf| {
            // protocol version: major = 3, minor = 0
            buf.extend_from_slice(&0x0003_i16.to_be_bytes());
            buf.extend_from_slice(&0x0000_i16.to_be_bytes());

            for (name, value) in self.params {
                if let Some(value) = value {
                    write_str(buf, name);
                    write_str(buf, value);
                }
            }

            // A zero byte is required as a terminator after the last name/value pair.
            buf.push(0);
        });
    }
}

#[derive(Debug)]
pub struct PasswordMessage<'a> {
    pub password: &'a str,
}

impl<'a> Serialize for PasswordMessage<'a> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(b'p');

        with_length_prefix(buf, |buf| {
            write_str(buf, self.password);
        });
    }
}

#[derive(Debug)]
pub struct Query<'a> {
    pub query: &'a str,
}

impl<'a> Serialize for Query<'a> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(b'Q');

        with_length_prefix(buf, |buf| {
            write_str(buf, self.query);
        });
    }
}

// Write a string followed by a null-terminator
#[inline]
fn write_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    // TODO: encode test more messages
//
//    #[test]
//    fn ser_startup_message() {
//        let msg = StartupMessage { user: "postgres", database: None };
//
//        let mut buf = Vec::new();
//        msg.encode(&mut buf);
//
//        assert_eq!(
//            "00000074000300007573657200706f7374677265730044617465537\
//             4796c650049534f00496e74657276616c5374796c650069736f5f38\
//             3630310054696d655a6f6e65005554430065787472615f666c6f617\
//             45f646967697473003300636c69656e745f656e636f64696e670055\
//             54462d380000",
//            hex::encode(buf)
//        );
//    }
//}
