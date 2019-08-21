use super::{
    protocol::{self, BufMut},
    Pg, PgRawConnection,
};
use crate::{
    query::RawQuery,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder};

pub struct PgRawQuery<'q> {
    limit: i32,
    query: &'q str,
    // OIDs of the bind parameters
    types: Vec<u32>,
    // Write buffer for serializing bind values
    buf: Vec<u8>,
}

impl<'q> RawQuery<'q> for PgRawQuery<'q> {
    type Backend = Pg;

    fn new(query: &'q str) -> Self {
        Self {
            limit: 0,
            query,
            // Estimates for average number of bind parameters were
            // chosen from sampling from internal projects
            types: Vec::with_capacity(4),
            buf: Vec::with_capacity(32),
        }
    }

    fn bind_as<ST, T>(mut self, value: T) -> Self
    where
        Self: Sized,
        Self::Backend: HasSqlType<ST>,
        T: ToSql<ST, Self::Backend>,
    {
        // TODO: When/if we receive types that do _not_ support BINARY, we need to check here
        // TODO: There is no need to be explicit unless we are expecting mixed BINARY / TEXT

        self.types.push(<Pg as HasSqlType<ST>>::metadata().oid);

        let pos = self.buf.len();
        self.buf.put_int_32(0);

        let len = if let IsNull::No = value.to_sql(&mut self.buf) {
            (self.buf.len() - pos - 4) as i32
        } else {
            // Write a -1 for the len to indicate NULL
            // TODO: It is illegal for [to_sql] to write any data if IsSql::No; fail a debug assertion
            -1
        };

        // Write-back the len to the beginning of this frame (not including the len of len)
        BigEndian::write_i32(&mut self.buf[pos..], len as i32);

        self
    }

    fn finish(self, conn: &mut PgRawConnection) {
        conn.write(protocol::Parse {
            portal: "",
            query: self.query,
            param_types: &*self.types,
        });

        conn.write(protocol::Bind {
            portal: "",
            statement: "",
            formats: &[1], // [BINARY]
            // TODO: Early error if there is more than i16
            values_len: self.types.len() as i16,
            values: &*self.buf,
            result_formats: &[1], // [BINARY]
        });

        // TODO: Make limit be 1 for fetch_optional
        conn.write(protocol::Execute {
            portal: "",
            limit: self.limit,
        });

        conn.write(protocol::Sync);
    }
}
