use crate::decode::DecodeOwned;
use crate::encode::{Encode, IsNull};
use crate::io::{Buf, BufMut};
use crate::postgres::types::raw::sequence::PgSequenceDecoder;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;
use byteorder::BE;
use std::convert::TryInto;
use std::marker::PhantomData;

// https://git.postgresql.org/gitweb/?p=postgresql.git;a=blob;f=src/include/utils/array.h;h=7f7e744cb12bc872f628f90dad99dfdf074eb314;hb=master#l6
// https://git.postgresql.org/gitweb/?p=postgresql.git;a=blob;f=src/backend/utils/adt/arrayfuncs.c;h=7a4a5aaa86dc1c8cffa2d899c89511dc317d485b;hb=master#l1547

pub(crate) struct PgArrayEncoder<'enc, T> {
    count: usize,
    len_start_index: usize,
    buf: &'enc mut Vec<u8>,
    phantom: PhantomData<T>,
}

impl<'enc, T> PgArrayEncoder<'enc, T>
where
    T: Encode<Postgres>,
    T: Type<Postgres>,
{
    pub(crate) fn new(buf: &'enc mut Vec<u8>) -> Self {
        let ty = <T as Type<Postgres>>::type_info();

        // ndim
        buf.put_i32::<BE>(1);

        // dataoffset
        buf.put_i32::<BE>(0);

        // elemtype
        buf.put_i32::<BE>(ty.id.0 as i32);
        let len_start_index = buf.len();

        // dimensions
        buf.put_i32::<BE>(0);

        // lower_bnds
        buf.put_i32::<BE>(1);

        Self {
            count: 0,
            len_start_index,
            buf,

            phantom: PhantomData,
        }
    }

    pub(crate) fn encode(&mut self, item: T) {
        // Allocate space for the length of the encoded elemement up front
        let el_len_index = self.buf.len();
        self.buf.put_i32::<BE>(0);

        // Allocate and encode the element it self
        let el_start = self.buf.len();

        if let IsNull::Yes = Encode::<Postgres>::encode_nullable(&item, self.buf) {
            self.buf[el_len_index..el_start].copy_from_slice(&(-1_i32).to_be_bytes());
        } else {
            let el_end = self.buf.len();

            // Now we know the actual length of the encoded element
            let el_len = el_end - el_start;

            // And we can now go back and update the length
            self.buf[el_len_index..el_start].copy_from_slice(&(el_len as i32).to_be_bytes());
        }

        self.count += 1;
    }

    pub(crate) fn finish(&mut self) {
        const I32_SIZE: usize = std::mem::size_of::<i32>();

        let size_bytes = (self.count as i32).to_be_bytes();

        self.buf[self.len_start_index..self.len_start_index + I32_SIZE]
            .copy_from_slice(&size_bytes);
    }
}

pub(crate) struct PgArrayDecoder<'de, T> {
    inner: PgSequenceDecoder<'de>,
    phantom: PhantomData<T>,
}

impl<'de, T> PgArrayDecoder<'de, T>
where
    T: DecodeOwned<Postgres>,
    T: Type<Postgres>,
{
    pub(crate) fn new(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        let mut value = value.try_into()?;

        match value {
            PgValue::Binary(ref mut buf) => {
                // number of dimensions of the array
                let ndim = buf.get_i32::<BE>()?;

                if ndim == 0 {
                    return Ok(Self {
                        inner: PgSequenceDecoder::new(PgValue::Binary(&[]), false),
                        phantom: PhantomData,
                    });
                }

                if ndim != 1 {
                    return Err(decode_err!(
                        "encountered an array of {} dimensions; only one-dimensional arrays are supported",
                        ndim
                    ));
                }

                // offset to stored data
                // this doesn't matter as the data is always at the end of the header
                let _dataoffset = buf.get_i32::<BE>()?;

                // TODO: Validate element type with whatever framework is put in place to do so
                //       As a reminder, we have no way to do this yet and still account for [compatible]
                //       types.

                // element type OID
                let _elemtype = buf.get_i32::<BE>()?;

                // length of each array axis
                let _dimensions = buf.get_i32::<BE>()?;

                // lower boundary of each dimension
                let lower_bnds = buf.get_i32::<BE>()?;

                if lower_bnds != 1 {
                    return Err(decode_err!(
                        "encountered an array with a lower bound of {} in the first dimension; only arrays starting at one are supported",
                        lower_bnds
                    ));
                }
            }

            PgValue::Text(_) => {}
        }

        Ok(Self {
            inner: PgSequenceDecoder::new(value, false),
            phantom: PhantomData,
        })
    }

    fn decode(&mut self) -> crate::Result<Option<T>> {
        self.inner.decode()
    }
}

impl<'de, T> Iterator for PgArrayDecoder<'de, T>
where
    T: 'de,
    T: DecodeOwned<Postgres>,
    T: Type<Postgres>,
{
    type Item = crate::Result<T>;

    #[inline]
    fn next(&mut self) -> Option<crate::Result<T>> {
        self.decode().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::PgArrayDecoder;
    use super::PgArrayEncoder;
    use crate::postgres::PgValue;

    const BUF_BINARY_I32: &[u8] = b"\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x17\x00\x00\x00\x04\x00\x00\x00\x01\x00\x00\x00\x04\x00\x00\x00\x01\x00\x00\x00\x04\x00\x00\x00\x02\x00\x00\x00\x04\x00\x00\x00\x03\x00\x00\x00\x04\x00\x00\x00\x04";

    #[test]
    fn it_encodes_i32() {
        let mut buf = Vec::new();
        let mut encoder = PgArrayEncoder::new(&mut buf);

        for val in &[1_i32, 2, 3, 4] {
            encoder.encode(*val);
        }

        encoder.finish();

        assert_eq!(buf, BUF_BINARY_I32);
    }

    #[test]
    fn it_decodes_text_i32() -> crate::Result<()> {
        let s = "{1,152,-12412}";
        let mut decoder = PgArrayDecoder::<i32>::new(Some(PgValue::Text(s)))?;

        assert_eq!(decoder.decode()?, Some(1));
        assert_eq!(decoder.decode()?, Some(152));
        assert_eq!(decoder.decode()?, Some(-12412));
        assert_eq!(decoder.decode()?, None);

        Ok(())
    }

    #[test]
    fn it_decodes_text_str() -> crate::Result<()> {
        let s = "{\"\",\"\\\"\"}";
        let mut decoder = PgArrayDecoder::<String>::new(Some(PgValue::Text(s)))?;

        assert_eq!(decoder.decode()?, Some("".to_string()));
        assert_eq!(decoder.decode()?, Some("\"".to_string()));
        assert_eq!(decoder.decode()?, None);

        Ok(())
    }

    #[test]
    fn it_decodes_binary_nulls() -> crate::Result<()> {
        let mut decoder = PgArrayDecoder::<Option<bool>>::new(Some(PgValue::Binary(
            b"\x00\x00\x00\x01\x00\x00\x00\x01\x00\x00\x00\x10\x00\x00\x00\x04\x00\x00\x00\x01\xff\xff\xff\xff\x00\x00\x00\x01\x01\xff\xff\xff\xff\x00\x00\x00\x01\x00"
        )))?;

        assert_eq!(decoder.decode()?, Some(None));
        assert_eq!(decoder.decode()?, Some(Some(true)));
        assert_eq!(decoder.decode()?, Some(None));
        assert_eq!(decoder.decode()?, Some(Some(false)));

        Ok(())
    }

    #[test]
    fn it_decodes_binary_i32() -> crate::Result<()> {
        let mut decoder = PgArrayDecoder::<i32>::new(Some(PgValue::Binary(BUF_BINARY_I32)))?;

        let val_1 = decoder.decode()?;
        let val_2 = decoder.decode()?;
        let val_3 = decoder.decode()?;
        let val_4 = decoder.decode()?;

        assert_eq!(val_1, Some(1));
        assert_eq!(val_2, Some(2));
        assert_eq!(val_3, Some(3));
        assert_eq!(val_4, Some(4));

        assert!(decoder.decode()?.is_none());

        Ok(())
    }
}
