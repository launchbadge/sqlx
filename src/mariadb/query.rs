use super::MariaDb;
use crate::{
    query::QueryParameters,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use crate::mariadb::types::MariaDbTypeMetadata;

pub struct MariaDbQueryParameters {
    param_types: Vec<MariaDbTypeMetadata>,
    params: Vec<u8>,
    null: Vec<u8>,
}

impl QueryParameters for MariaDbQueryParameters {
    type Backend = MariaDb;

    fn new() -> Self {
        Self {
            param_types: Vec::with_capacity(4),
            params: Vec::with_capacity(32),
            null: vec![0],
        }
    }

    fn bind<T>(&mut self, value: T)
    where
        Self: Sized,
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>,
    {
        let metadata = <MariaDb as HasSqlType<T>>::metadata();
        let index = self.param_types.len();

        self.param_types.push(metadata);

        if let IsNull::Yes = value.to_sql(&mut self.params) {
            self.null[index / 8] = self.null[index / 8] & (1 << index % 8);
        }
    }
}
