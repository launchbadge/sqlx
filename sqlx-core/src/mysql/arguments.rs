use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::MySql;
use crate::types::Type;

#[derive(Default)]
pub struct MySqlArguments {
    pub(crate) param_types: Vec<MySqlTypeInfo>,
    pub(crate) params: Vec<u8>,
    pub(crate) null_bitmap: Vec<u8>,
}

impl Arguments for MySqlArguments {
    type Database = MySql;

    fn reserve(&mut self, len: usize, size: usize) {
        self.param_types.reserve(len);
        self.params.reserve(size);

        // ensure we have enough size in the bitmap to hold at least `len` extra bits
        // the second `& 7` gives us 0 spare bits when param_types.len() is a multiple of 8
        let spare_bits = (8 - (self.param_types.len()) & 7) & 7;
        // ensure that if there are no spare bits left, `len = 1` reserves another byte
        self.null_bitmap.reserve((len + 7 - spare_bits) / 8);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Type<Self::Database>,
        T: Encode<Self::Database>,
    {
        let type_id = <T as Type<MySql>>::type_info();
        let index = self.param_types.len();

        self.param_types.push(type_id);
        self.null_bitmap.resize((index / 8) + 1, 0);

        if let IsNull::Yes = value.encode_nullable(&mut self.params) {
            self.null_bitmap[index / 8] |= (1 << index % 8) as u8;
        }
    }
}
