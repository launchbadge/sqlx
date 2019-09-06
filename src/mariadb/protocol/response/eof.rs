use crate::{
    io::Buf,
    mariadb::{
        io::BufExt,
        protocol::{ErrorCode, ServerStatusFlag},
    },
};
use byteorder::LittleEndian;
use std::io;

#[derive(Debug)]
pub struct EofPacket {
    pub warning_count: u16,
    pub status: ServerStatusFlag,
}

impl EofPacket {
    fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let header = buf.get_u8()?;
        if header != 0xFE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected 0xFE; received {}", header),
            ));
        }

        let warning_count = buf.get_u16::<LittleEndian>()?;
        let status = ServerStatusFlag::from_bits_truncate(buf.get_u16::<LittleEndian>()?);

        Ok(Self {
            warning_count,
            status,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;
    use std::io;

    #[test]
    fn it_decodes_eof_packet() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // int<1> 0xfe : EOF header
            0xFE_u8,
            // int<2> warning count
            0u8, 0u8,
            // int<2> server status
            1u8, 1u8
        );

        let _message = EofPacket::decode(&buf)?;

        // TODO: Assert fields?

        Ok(())
    }
}
