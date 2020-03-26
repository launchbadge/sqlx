use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::io::Buf;
use crate::postgres::types::raw::sequence::PgSequenceDecoder;
use crate::postgres::{PgData, PgValue, Postgres};
use crate::types::Type;
use byteorder::BigEndian;

pub struct PgRecordEncoder<'a> {
    buf: &'a mut Vec<u8>,
    beg: usize,
    num: u32,
}

impl<'a> PgRecordEncoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        // reserve space for a field count
        buf.extend_from_slice(&(0_u32).to_be_bytes());

        Self {
            beg: buf.len(),
            buf,
            num: 0,
        }
    }

    pub fn finish(&mut self) {
        // replaces zeros with actual length
        self.buf[self.beg - 4..self.beg].copy_from_slice(&self.num.to_be_bytes());
    }

    pub fn encode<T>(&mut self, value: T) -> &mut Self
    where
        T: Type<Postgres> + Encode<Postgres>,
    {
        // write oid
        let info = T::type_info();
        self.buf.extend(&info.oid().to_be_bytes());

        // write zeros for length
        self.buf.extend(&[0; 4]);

        let start = self.buf.len();
        if let IsNull::Yes = value.encode_nullable(self.buf) {
            self.buf[start - 4..start].copy_from_slice(&(-1_i32).to_be_bytes());
        } else {
            let end = self.buf.len();
            let size = end - start;

            // replaces zeros with actual length
            self.buf[start - 4..start].copy_from_slice(&(size as u32).to_be_bytes());
        }

        // keep track of count
        self.num += 1;

        self
    }
}

pub struct PgRecordDecoder<'de>(PgSequenceDecoder<'de>);

impl<'de> PgRecordDecoder<'de> {
    pub fn new(value: PgValue<'de>) -> crate::Result<Self> {
        let mut data = value.try_get()?;

        match data {
            PgData::Text(_) => {}
            PgData::Binary(ref mut buf) => {
                let _expected_len = buf.get_u32::<BigEndian>()?;
            }
        }

        Ok(Self(PgSequenceDecoder::new(data, true)))
    }

    #[inline]
    pub fn decode<T>(&mut self) -> crate::Result<T>
    where
        T: for<'rec> Decode<'rec, Postgres>,
        T: Type<Postgres>,
    {
        self.0
            .decode()?
            .ok_or_else(|| decode_err!("no field `{0}` on {0}-element record", self.0.len()))
    }
}

#[test]
fn test_encode_field() {
    use std::convert::TryInto;

    let value = "Foo Bar";
    let mut raw_encoded = Vec::new();
    <&str as Encode<Postgres>>::encode(&value, &mut raw_encoded);
    let mut field_encoded = Vec::new();

    let mut encoder = PgRecordEncoder::new(&mut field_encoded);
    encoder.encode(&value);

    // check oid
    let oid = <&str as Type<Postgres>>::type_info().oid();
    let field_encoded_oid = u32::from_be_bytes(field_encoded[4..8].try_into().unwrap());
    assert_eq!(oid, field_encoded_oid);

    // check length
    let field_encoded_length = u32::from_be_bytes(field_encoded[8..12].try_into().unwrap());
    assert_eq!(raw_encoded.len(), field_encoded_length as usize);

    // check data
    assert_eq!(raw_encoded, &field_encoded[12..]);
}

#[test]
fn test_decode_field() {
    use crate::postgres::protocol::TypeId;

    let value = "Foo Bar".to_string();

    let mut buf = Vec::new();
    let mut encoder = PgRecordEncoder::new(&mut buf);
    encoder.encode(&value);

    let buf = buf.as_slice();
    let mut decoder = PgRecordDecoder::new(PgValue::bytes(TypeId(0), buf)).unwrap();

    let value_decoded: String = decoder.decode().unwrap();
    assert_eq!(value_decoded, value);
}
