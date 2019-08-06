use super::{BufMut, Encode};
use crate::{
    postgres::Postgres,
    serialize::ToSql,
    types::{AsSql, SqlType},
};
use byteorder::{BigEndian, ByteOrder};

const TEXT: i16 = 0;
const BINARY: i16 = 1;

// FIXME: Think of a better name here
pub struct BindValues {
    types: Vec<i32>,
    formats: Vec<i16>,
    values_len: i16,
    values: Vec<u8>,
}

impl BindValues {
    pub fn new() -> Self {
        BindValues {
            types: Vec::new(),
            formats: Vec::new(),
            values: Vec::new(),
            values_len: 0,
        }
    }

    #[inline]
    pub fn add<T: AsSql<Postgres>>(&mut self, value: T)
    where
        T: ToSql<Postgres, <T as AsSql<Postgres>>::Type>,
    {
        self.add_as::<T::Type, T>(value);
    }

    pub fn add_as<ST: SqlType<Postgres>, T: ToSql<Postgres, ST>>(&mut self, value: T) {
        // TODO: When/if we receive types that do _not_ support BINARY, we need to check here
        // TODO: There is no need to be explicit unless we are expecting mixed BINARY / TEXT

        self.types.push(ST::metadata().oid as i32);

        let pos = self.values.len();
        self.values.put_int_32(0); // skip over len

        value.to_sql(&mut self.values);
        self.values_len += 1;

        // Write-back the len to the beginning of this frame (not including the len of len)
        let len = self.values.len() - pos - 4;
        BigEndian::write_i32(&mut self.values[pos..], len as i32);
    }

    pub fn types(&self) -> &[i32] {
        &self.types
    }

    pub fn formats(&self) -> &[i16] {
        // &self.formats
        &[BINARY]
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    pub fn values_len(&self) -> i16 {
        self.values_len
    }
}

pub struct Bind<'a> {
    /// The name of the destination portal (an empty string selects the unnamed portal).
    pub portal: &'a str,

    /// The name of the source prepared statement (an empty string selects the unnamed prepared statement).
    pub statement: &'a str,

    /// The parameter format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no parameters or that the parameters all use the
    /// default format (text); or one, in which case the specified format code is applied to all
    /// parameters; or it can equal the actual number of parameters.
    pub formats: &'a [i16],

    pub values_len: i16,
    pub values: &'a [u8],

    /// The result-column format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no result columns or that the
    /// result columns should all use the default format (text); or one, in which
    /// case the specified format code is applied to all result columns (if any);
    /// or it can equal the actual number of result columns of the query.
    pub result_formats: &'a [i16],
}

impl Encode for Bind<'_> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_byte(b'B');

        let pos = buf.len();
        buf.put_int_32(0); // skip over len

        buf.put_str(self.portal);
        buf.put_str(self.statement);

        buf.put_array_int_16(&self.formats);

        buf.put_int_16(self.values_len);

        buf.put(self.values);

        buf.put_array_int_16(&self.result_formats);

        // Write-back the len to the beginning of this frame
        let len = buf.len() - pos;
        BigEndian::write_i32(&mut buf[pos..], len as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::{Bind, BindValues, BufMut, Encode};

    const BIND: &[u8] = b"B\0\0\0\x18\0\0\0\x01\0\x01\0\x02\0\0\0\x011\0\0\0\x012\0\0";

    #[test]
    fn it_encodes_bind_for_two() {
        let mut buf = Vec::new();

        let mut builder = BindValues::new();
        builder.add("1");
        builder.add("2");

        let bind = Bind {
            portal: "",
            statement: "",
            formats: builder.formats(),
            values_len: builder.values_len(),
            values: builder.values(),
            result_formats: &[],
        };

        bind.encode(&mut buf);

        assert_eq!(buf, BIND);
    }
}
