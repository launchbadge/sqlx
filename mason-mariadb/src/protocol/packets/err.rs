use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use super::super::{decode::Decoder, deserialize::Deserialize, error_codes::ErrorCode};

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
    fn deserialize(decoder: &mut Decoder) -> Result<Self, Error> {
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
                error_message = Some(decoder.decode_string_eof());
            } else {
                error_message = Some(decoder.decode_string_eof());
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

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn it_decodes_errpacket() -> Result<(), Error> {
        let buf = BytesMut::from(b"!\0\0\x01\xff\x84\x04#08S01Got packets out of order".to_vec());
        let _message = ErrPacket::deserialize(&mut Decoder::new(&buf.freeze()))?;

        Ok(())
    }
}
