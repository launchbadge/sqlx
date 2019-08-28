use crate::{
    mariadb::{protocol::types::ParamFlag, FieldType, MariaDbRawConnection},
    query::RawQuery,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};

pub struct MariaDbRawQuery<'q> {
    query: &'q str,
    types: Vec<u8>,
    null_bitmap: Vec<u8>,
    flags: Vec<u8>,
    buf: Vec<u8>,
    index: u64,
}

impl<'q> RawQueryQuery<'q> for MariaDbRawQuery<'q> {
    type Backend = MariaDb;

    fn new(query: &'q str) -> Self {
        Self {
            query,
            types: Vec::with_capacity(4),
            null_bitmap: vec![0, 0, 0, 0],
            flags: Vec::with_capacity(4),
            buf: Vec::with_capacity(32),
            index: 0,
        }
    }

    fn bind<T>(mut self, value: T) -> Self
    where
        Self: Sized,
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>,
    {
        self.types
            .push(<MariaDb as HasSqlType<T>>::metadata().field_type.0);
        self.flags
            .push(<MariaDb as HasSqlType<T>>::metadata().param_flag.0);

        match value.to_sql(&mut self.buf) {
            IsNull::Yes => {
                self.null_bitmap[self.index / 8] =
                    self.null_bitmap[self.index / 8] & (1 << self.index % 8);
            }
            IsNull::No => {}
        }

        self
    }

    fn finish(self, conn: &mut MariaDbRawConnection) {
        conn.prepare(self.query);
    }
}
