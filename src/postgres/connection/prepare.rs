use super::Connection;
use crate::{
    postgres::protocol::{self, Parse},
    types::ToSql,
};

pub struct Prepare<'a> {
    pub(super) connection: &'a mut Connection,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut Connection, query: &'b str) -> Prepare<'a> {
    // TODO: Use a hash map to cache the parse
    // TODO: Use named statements
    connection.write(Parse {
        portal: "",
        query,
        param_types: &[],
    });

    Prepare { connection }
}

// impl<'a> Prepare<'a> {
//     #[inline]
//     pub fn bind<T>(mut self, value: impl ToSql<T>) -> Self {
//         unimplemented!()
//     }
// }
