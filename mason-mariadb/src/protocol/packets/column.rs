use super::super::{decode::Decoder, deserialize::Deserialize};
use failure::Error;

#[derive(Default, Debug)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<usize>,
}

impl Deserialize for ColumnPacket {
    fn deserialize(decoder: &mut Decoder) -> Result<Self, Error> {
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();
        let columns = decoder.decode_int_lenenc();

        Ok(ColumnPacket { length, seq_no, columns })
    }
}
