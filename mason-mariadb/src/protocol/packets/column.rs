use super::super::{decode::Decoder, deserialize::Deserialize};
use bytes::Bytes;
use failure::Error;

#[derive(Default, Debug)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<usize>,
}

impl Deserialize for ColumnPacket {
    fn deserialize<'a, 'b>(
        buf: &'a Bytes,
        decoder: Option<&'b mut Decoder<'a>>,
    ) -> Result<Self, Error> {
        let mut new_decoder = Decoder::new(&buf);
        let mut decoder = decoder.unwrap_or(&mut new_decoder);

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();
        let columns = decoder.decode_int_lenenc();

        Ok(ColumnPacket { length, seq_no, columns })
    }
}
