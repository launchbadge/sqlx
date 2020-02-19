use byteorder::{ByteOrder, NetworkEndian};

use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::io::BufMut;
use crate::types::HasSqlType;
use crate::Postgres;

#[derive(Default)]
pub struct PgArguments {
    // OIDs of the bind parameters
    pub(super) types: Vec<u32>,

    // Write buffer for serializing bind values
    pub(super) values: Vec<u8>,
}

impl Arguments for PgArguments {
    type Database = super::Postgres;

    fn reserve(&mut self, len: usize, size: usize) {
        self.types.reserve(len);
        self.values.reserve(size);
    }

    fn add<T>(&mut self, value: T)
    where
        Self::Database: HasSqlType<T>,
        T: Encode<Self::Database>,
    {
        // TODO: When/if we receive types that do _not_ support BINARY, we need to check here
        // TODO: There is no need to be explicit unless we are expecting mixed BINARY / TEXT

        self.types
            .push(<Postgres as HasSqlType<T>>::type_info().id.0);

        let pos = self.values.len();

        self.values.put_i32::<NetworkEndian>(0);

        let len = if let IsNull::No = value.encode_nullable(&mut self.values) {
            (self.values.len() - pos - 4) as i32
        } else {
            // Write a -1 for the len to indicate NULL
            // TODO: It is illegal for [encode] to write any data
            //       if IsSql::No; fail a debug assertion
            -1
        };

        // Write-back the len to the beginning of this frame (not including the len of len)
        NetworkEndian::write_i32(&mut self.values[pos..], len as i32);
    }
}
