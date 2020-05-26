use std::borrow::Cow;

use atoi::atoi;
use libsqlite3_sys::SQLITE_OK;

use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::error::Error;
use crate::sqlite::statement::{SqliteStatement, StatementHandle};
use crate::sqlite::Sqlite;

#[derive(Debug, Clone)]
pub enum SqliteArgumentValue<'q> {
    Null,
    Text(Cow<'q, str>),
    Blob(Cow<'q, [u8]>),
    Double(f64),
    Int(i32),
    Int64(i64),
}

#[derive(Default)]
pub struct SqliteArguments<'q> {
    pub(crate) values: Vec<SqliteArgumentValue<'q>>,
}

impl<'q> Arguments<'q> for SqliteArguments<'q> {
    type Database = Sqlite;

    fn reserve(&mut self, len: usize, _size_hint: usize) {
        self.values.reserve(len);
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Encode<'q, Self::Database>,
    {
        if let IsNull::Yes = value.encode(&mut self.values) {
            self.values.push(SqliteArgumentValue::Null);
        }
    }
}

impl SqliteArguments<'_> {
    pub(super) fn bind(&self, statement: &SqliteStatement) -> Result<(), Error> {
        let mut arg_i = 0;
        for handle in &statement.handles {
            let cnt = handle.bind_parameter_count();
            for param_i in 0..cnt {
                // figure out the index of this bind parameter into our argument tuple
                let n: usize = if let Some(name) = handle.bind_parameter_name(param_i) {
                    if name.starts_with('?') {
                        // parameter should have the form ?NNN
                        atoi(name[1..].as_bytes()).expect("parameter of the form ?NNN")
                    } else {
                        return Err(err_protocol!("unsupported SQL parameter format: {}", name));
                    }
                } else {
                    arg_i += 1;
                    arg_i
                };

                if n > self.values.len() {
                    return Err(err_protocol!(
                        "wrong number of parameters, parameter ?{} requested but have only {}",
                        n,
                        self.values.len()
                    ));
                }

                self.values[n - 1].bind(handle, param_i + 1)?;
            }
        }

        Ok(())
    }
}

impl SqliteArgumentValue<'_> {
    fn bind(&self, handle: &StatementHandle, i: usize) -> Result<(), Error> {
        use SqliteArgumentValue::*;

        let status = match self {
            Text(v) => handle.bind_text(i, v),
            Blob(v) => handle.bind_blob(i, v),
            Int(v) => handle.bind_int(i, *v),
            Int64(v) => handle.bind_int64(i, *v),
            Double(v) => handle.bind_double(i, *v),
            Null => handle.bind_null(i),
        };

        if status != SQLITE_OK {
            return Err(handle.last_error().into());
        }

        Ok(())
    }
}
