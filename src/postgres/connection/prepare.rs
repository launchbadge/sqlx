use super::Connection;
use crate::postgres::protocol::{self, Parse};

pub struct Prepare<'a> {
    pub(super) connection: &'a mut Connection,
    pub(super) bind_state: (usize, usize),
    pub(super) bind_values: usize,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut Connection, query: &'b str) -> Prepare<'a> {
    // TODO: Use a hash map to cache the parse
    // TODO: Use named statements
    connection.send(Parse::new("", query, &[]));

    let bind_state = protocol::bind::header(&mut connection.wbuf, "", "", &[]);

    Prepare {
        connection,
        bind_state,
        bind_values: 0,
    }
}

impl<'a> Prepare<'a> {
    #[inline]
    pub fn bind<'b>(mut self, value: &'b [u8]) -> Self {
        protocol::bind::value(&mut self.connection.wbuf, value);
        self.bind_values += 1;
        self
    }

    #[inline]
    pub fn bind_null<'b>(mut self) -> Self {
        protocol::bind::value_null(&mut self.connection.wbuf);
        self.bind_values += 1;
        self
    }
}
