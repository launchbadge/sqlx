use crate::{
    io::Buf,
    mariadb::{error::Error, io::BufExt, protocol::ErrorCode},
};
use byteorder::LittleEndian;
use std::io;

#[derive(Debug)]
pub enum ErrPacket {
    Progress {
        stage: u8,
        max_stage: u8,
        progress: u32,
        info: Box<str>,
    },

    Error {
        code: ErrorCode,
        sql_state: Option<Box<str>>,
        message: Box<str>,
    },
}

impl ErrPacket {
    pub fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let header = buf.get_u8()?;
        debug_assert_eq!(header, 0xFF);

        // error code : int<2>
        let code = buf.get_u16::<LittleEndian>()?;

        // if (errorcode == 0xFFFF) /* progress reporting */
        if code == 0xFF_FF {
            let stage = buf.get_u8()?;
            let max_stage = buf.get_u8()?;
            let progress = buf.get_u24::<LittleEndian>()?;
            let info = buf
                .get_str_lenenc::<LittleEndian>()?
                .unwrap_or_default()
                .into();

            Ok(Self::Progress {
                stage,
                max_stage,
                progress,
                info,
            })
        } else {
            // if (next byte = '#')
            let sql_state = if buf[0] == b'#' {
                // '#' : string<1>
                buf.advance(1);

                // sql state : string<5>
                Some(buf.get_str(5)?.into())
            } else {
                None
            };

            let message = buf.get_str_eof()?.into();

            Ok(Self::Error {
                code: ErrorCode(code),
                sql_state,
                message,
            })
        }
    }

    pub fn expect_error<T>(self) -> crate::Result<T> {
        match self {
            ErrPacket::Progress { .. } => {
                Err(protocol_err!("expected ErrPacket::Err, got {:?}", self).into())
            }
            ErrPacket::Error { code, message, .. } => Err(Error { code, message }.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_err_packet() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // int<1> 0xfe : EOF header
            0xFF_u8,
            // int<2> error code
            0x84_u8, 0x04_u8,
            // if (errorcode == 0xFFFF) /* progress reporting */ {
            //     int<1> stage
            //     int<1> max_stage
            //     int<3> progress
            //     string<lenenc> progress_info
            // } else {
            //     if (next byte = '#') {
            //         string<1> sql state marker '#'
                        b"#",
            //         string<5>sql state
                        b"08S01",
            //         string<EOF> error message
                        b"Got packets out of order"
            //     } else {
            //         string<EOF> error message
            //     }
            // }
        );

        let _message = ErrPacket::decode(&buf)?;

        Ok(())
    }
}
