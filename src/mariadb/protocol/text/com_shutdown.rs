use crate::{io::BufMut, mariadb::Encode};

#[derive(Clone, Copy)]
pub enum ShutdownOptions {
    ShutdownDefault = 0x00,
}

pub struct ComShutdown {
    pub option: ShutdownOptions,
}

impl Encode for ComShutdown {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComShutdown as u8);
        buf.put_u8(self.option as u8);
    }
}

// Helper method to easily transform into u8
impl Into<u8> for ShutdownOptions {
    fn into(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_shutdown() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComShutdown {
            option: ShutdownOptions::ShutdownDefault,
        }
        .encode(&mut buf);

        assert_eq!(&buf[..], b"\x02\0\0\x00\x0A\x00");

        Ok(())
    }
}
