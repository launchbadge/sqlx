/// Encoding and decoding of Postgres arrays. Documentation of the byte format can be found [here](https://git.postgresql.org/gitweb/?p=postgresql.git;a=blob;f=src/include/utils/array.h;h=7f7e744cb12bc872f628f90dad99dfdf074eb314;hb=master#l6)
use crate::decode::Decode;
use crate::decode::DecodeError;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::postgres::database::Postgres;
use crate::types::HasSqlType;
use PhantomData;

impl<T> Encode<Postgres> for [T]
where
    T: Encode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        let mut encoder = ArrayEncoder::new(buf);
        for item in self {
            encoder.push(item);
        }
    }
}
impl<T> Encode<Postgres> for Vec<T>
where
    [T]: Encode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        self.as_slice().encode(buf)
    }
}

impl<T> Decode<Postgres> for Vec<T>
where
    T: Decode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let decoder = ArrayDecoder::<T>::new(buf)?;
        decoder.collect()
    }
}

type Order = byteorder::BigEndian;

struct ArrayDecoder<'a, T>
where
    T: Decode<Postgres>,
    Postgres: HasSqlType<T>,
{
    left: usize,
    did_error: bool,

    buf: &'a [u8],

    phantom: PhantomData<T>,
}

impl<T> ArrayDecoder<'_, T>
where
    T: Decode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn new(mut buf: &[u8]) -> Result<ArrayDecoder<T>, DecodeError> {
        let ndim = buf.get_i32::<Order>()?;
        let dataoffset = buf.get_i32::<Order>()?;
        let elemtype = buf.get_i32::<Order>()?;

        if ndim == 0 {
            return Ok(ArrayDecoder {
                left: 0,
                did_error: false,
                buf,
                phantom: PhantomData,
            });
        }

        assert_eq!(ndim, 1, "only arrays of dimension 1 is supported");

        let dimensions = buf.get_i32::<Order>()?;
        let lower_bnds = buf.get_i32::<Order>()?;

        assert_eq!(dataoffset, 0, "arrays with [null bitmap] is not supported");
        assert_eq!(
            elemtype,
            <Postgres as HasSqlType<T>>::type_info().id.0 as i32,
            "mismatched array element type"
        );
        assert_eq!(lower_bnds, 1);

        Ok(ArrayDecoder {
            left: dimensions as usize,
            did_error: false,
            buf,

            phantom: PhantomData,
        })
    }

    /// Decodes the next element without worring how many are left, or if it previously errored
    fn decode_next_element(&mut self) -> Result<T, DecodeError> {
        let len = self.buf.get_i32::<Order>()?;
        let bytes = self.buf.get_bytes(len as usize)?;
        Decode::decode(bytes)
    }
}

impl<T> Iterator for ArrayDecoder<'_, T>
where
    T: Decode<Postgres>,
    Postgres: HasSqlType<T>,
{
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Result<T, DecodeError>> {
        if self.did_error || self.left == 0 {
            return None;
        }

        self.left -= 1;

        let decoded = self.decode_next_element();
        self.did_error = decoded.is_err();
        Some(decoded)
    }
}

struct ArrayEncoder<'a, T>
where
    T: Encode<Postgres>,
    Postgres: HasSqlType<T>,
{
    count: usize,
    len_start_index: usize,
    buf: &'a mut Vec<u8>,

    phantom: PhantomData<T>,
}

impl<T> ArrayEncoder<'_, T>
where
    T: Encode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn new(buf: &mut Vec<u8>) -> ArrayEncoder<T> {
        let ty = <Postgres as HasSqlType<T>>::type_info();

        // ndim
        buf.put_i32::<Order>(1);
        // dataoffset
        buf.put_i32::<Order>(0);
        // elemtype
        buf.put_i32::<Order>(ty.id.0 as i32);
        let len_start_index = buf.len();
        // dimensions
        buf.put_i32::<Order>(0);
        // lower_bnds
        buf.put_i32::<Order>(1);

        ArrayEncoder {
            count: 0,
            len_start_index,
            buf,

            phantom: PhantomData,
        }
    }
    fn push(&mut self, item: &T) {
        // Allocate space for the length of the encoded elemement up front
        let el_len_index = self.buf.len();
        self.buf.put_i32::<Order>(0);

        // Allocate the element it self
        let el_start = self.buf.len();
        Encode::encode(item, self.buf);
        let el_end = self.buf.len();

        // Now we know the actual length of the encoded element
        let el_len = el_end - el_start;

        // And we can now go back and update the length
        self.buf[el_len_index..el_start].copy_from_slice(&(el_len as i32).to_be_bytes());

        self.count += 1;
    }
    fn extend<'a, I>(&mut self, items: I)
    where
        I: Iterator<Item = &'a T>,
        T: 'a,
    {
        for item in items {
            self.push(item);
        }
    }
    fn update_len(&mut self) {
        const I32_SIZE: usize = std::mem::size_of::<i32>();

        let size_bytes = (self.count as i32).to_be_bytes();

        self.buf[self.len_start_index..self.len_start_index + I32_SIZE]
            .copy_from_slice(&size_bytes);
    }
}
impl<T> Drop for ArrayEncoder<'_, T>
where
    T: Encode<Postgres>,
    Postgres: HasSqlType<T>,
{
    fn drop(&mut self) {
        self.update_len();
    }
}
