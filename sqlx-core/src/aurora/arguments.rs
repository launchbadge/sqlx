use crate::arguments::Arguments;
use crate::aurora::Aurora;
use crate::encode::Encode;
use crate::types::Type;

use rusoto_rds_data::SqlParameter;

/// Implementation of [`Arguments`] for Aurora.
#[derive(Default)]
pub struct AuroraArguments {
    pub(crate) parameters: Vec<SqlParameter>,
}

impl AuroraArguments {
    pub(crate) fn add<'q, T>(&mut self, value: T)
    where
        T: Encode<'q, Aurora> + Type<Aurora>,
    {
        let _ = value.encode(&mut self.parameters);
    }
}

impl<'q> Arguments<'q> for AuroraArguments {
    type Database = Aurora;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.parameters.reserve(additional);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<'q, Self::Database> + Type<Self::Database>,
    {
        self.add(value);
    }
}
