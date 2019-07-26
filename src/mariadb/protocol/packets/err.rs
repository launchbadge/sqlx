use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use super::super::{
    deserialize::{DeContext, Deserialize},
    error_codes::ErrorCode,
};

#[derive(Default, Debug)]
pub struct ErrPacket {
    pub length: u32,
    pub seq_no: u8,
    pub error_code: ErrorCode,
    pub stage: Option<u8>,
    pub max_stage: Option<u8>,
    pub progress: Option<u32>,
    pub progress_info: Option<Bytes>,
    pub sql_state_marker: Option<Bytes>,
    pub sql_state: Option<Bytes>,
    pub error_message: Option<Bytes>,
}

impl Deserialize for ErrPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        let packet_header = decoder.decode_int_1();
        if packet_header != 0xFF {
            panic!("Packet header is not 0xFF for ErrPacket");
        }

        let error_code = ErrorCode::try_from(decoder.decode_int_2())?;

        let mut stage = None;
        let mut max_stage = None;
        let mut progress = None;
        let mut progress_info = None;

        let mut sql_state_marker = None;
        let mut sql_state = None;
        let mut error_message = None;

        // Progress Reporting
        if error_code as u16 == 0xFFFF {
            stage = Some(decoder.decode_int_1());
            max_stage = Some(decoder.decode_int_1());
            progress = Some(decoder.decode_int_3());
            progress_info = Some(decoder.decode_string_lenenc());
        } else {
            if decoder.buf[decoder.index] == b'#' {
                sql_state_marker = Some(decoder.decode_string_fix(1));
                sql_state = Some(decoder.decode_string_fix(5));
                error_message = Some(decoder.decode_string_eof(Some(length as usize)));
            } else {
                error_message = Some(decoder.decode_string_eof(Some(length as usize)));
            }
        }

        Ok(ErrPacket {
            length,
            seq_no,
            error_code,
            stage,
            max_stage,
            progress,
            progress_info,
            sql_state_marker,
            sql_state,
            error_message,
        })
    }
}

impl std::error::Error for ErrPacket {
    fn description(&self) -> &str {
        "Received error packet"
    }
}

impl std::fmt::Display for ErrPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}:{:?}", self.error_code, self.error_message)
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use mason_core::ConnectOptions;

    use crate::{__bytes_builder, connection::ConnContext, protocol::decode::Decoder};

    use super::*;

    #[runtime::test]
    async fn it_decodes_err_packet() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // int<3> length
            1u8, 0u8, 0u8,
            // int<1> seq_no
            1u8,
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

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, &buf);

        let _message = ErrPacket::deserialize(&mut ctx)?;

        Ok(())
    }
}
