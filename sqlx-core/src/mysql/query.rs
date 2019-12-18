use super::Connection;
use crate::{
    encode::{Encode, IsNull},
    mysql::types::MariaDbTypeMetadata,
    params::QueryParameters,
    types::HasSqlType,
};

#[derive(Default)]
pub struct MariaDbQueryParameters {
    pub(crate) param_types: Vec<MariaDbTypeMetadata>,
    pub(crate) params: Vec<u8>,
    pub(crate) null_bitmap: Vec<u8>,
}

impl QueryParameters for MariaDbQueryParameters {
    type Backend = Connection;

    fn reserve(&mut self, binds: usize, bytes: usize) {
        self.param_types.reserve(binds);
        self.params.reserve(bytes);

        // ensure we have enough bytes in the bitmap to hold at least `binds` extra bits
        // the second `& 7` gives us 0 spare bits when param_types.len() is a multiple of 8
        let spare_bits = (8 - (self.param_types.len()) & 7) & 7;
        // ensure that if there are no spare bits left, `binds = 1` reserves another byte
        self.null_bitmap.reserve( (binds + 7 - spare_bits) / 8);
    }

    fn bind<T>(&mut self, value: T)
    where
        Self: Sized,
        Self::Backend: HasSqlType<T>,
        T: Encode<Self::Backend>,
    {
        let metadata = <Connection as HasSqlType<T>>::metadata();
        let index = self.param_types.len();

        self.param_types.push(metadata);
        self.null_bitmap.resize((index / 8) + 1, 0);

        if let IsNull::Yes = value.encode(&mut self.params) {
            self.null_bitmap[index / 8] &= (1 << index % 8) as u8;
        }
    }
}
