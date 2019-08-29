use super::{protocol, Postgres, PostgresRawConnection};
use crate::{
    io::BufMut,
    query::QueryParameters,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder, NetworkEndian};

pub struct PostgresQueryParameters {
    // OIDs of the bind parameters
    pub(super) types: Vec<u32>,
    // Write buffer for serializing bind values
    pub(super) buf: Vec<u8>,
}

impl QueryParameters for PostgresQueryParameters {
    type Backend = Postgres;

    fn new() -> Self {
        Self {
            // Estimates for average number of bind parameters were
            // chosen from sampling from internal projects
            types: Vec::with_capacity(4),
            buf: Vec::with_capacity(32),
        }
    }

    fn bind<T>(&mut self, value: T)
    where
        Self: Sized,
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>,
    {
        // TODO: When/if we receive types that do _not_ support BINARY, we need to check here
        // TODO: There is no need to be explicit unless we are expecting mixed BINARY / TEXT

        self.types.push(<Postgres as HasSqlType<T>>::metadata().oid);

        let pos = self.buf.len();
        self.buf.put_i32::<NetworkEndian>(0);

        let len = if let IsNull::No = value.to_sql(&mut self.buf) {
            (self.buf.len() - pos - 4) as i32
        } else {
            // Write a -1 for the len to indicate NULL
            // TODO: It is illegal for [to_sql] to write any data if IsSql::No; fail a debug assertion
            -1
        };

        // Write-back the len to the beginning of this frame (not including the len of len)
        BigEndian::write_i32(&mut self.buf[pos..], len as i32);
    }
}
