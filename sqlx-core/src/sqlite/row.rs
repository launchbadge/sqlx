use core::str::{from_utf8, Utf8Error};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::error::UnexpectedNullError;
use crate::row::{ColumnIndex, Row};
use crate::sqlite::Sqlite;

pub struct SqliteRow<'c> {
    c: std::marker::PhantomData<&'c ()>,
}

impl<'c> Row<'c> for SqliteRow<'c> {
    type Database = Sqlite;

    fn len(&self) -> usize {
        todo!()
    }

    fn try_get_raw<'r, I>(&'r self, index: I) -> crate::Result<Option<()>>
    where
        I: ColumnIndex<Self::Database>,
    {
        todo!()
        // let index = index.resolve(self)?;
        // let buffer = self.data.get(index);
        //
        // buffer
        //     .map(|buf| match self.formats[index] {
        //         TypeFormat::Binary => Ok(PgValue::Binary(buf)),
        //         TypeFormat::Text => Ok(PgValue::Text(from_utf8(buf)?)),
        //     })
        //     .transpose()
        //     .map_err(|err: Utf8Error| crate::Error::Decode(Box::new(err)))
    }
}
