use std::ffi::{c_char, CString};
use std::os::raw::{c_int, c_void};
use std::sync::Arc;

use libsqlite3_sys::{
    sqlite3_context, sqlite3_create_function_v2, sqlite3_result_blob, sqlite3_result_double,
    sqlite3_result_error, sqlite3_result_int, sqlite3_result_int64, sqlite3_result_null,
    sqlite3_result_text, sqlite3_user_data, sqlite3_value,
    sqlite3_value_type, SQLITE_DETERMINISTIC, SQLITE_DIRECTONLY, SQLITE_OK,
    SQLITE_TRANSIENT, SQLITE_UTF8,
};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::sqlite::type_info::DataType;
use crate::sqlite::SqliteArgumentValue;
use crate::sqlite::SqliteTypeInfo;
use crate::sqlite::SqliteValue;
use crate::sqlite::Sqlite;
use crate::sqlite::{connection::handle::ConnectionHandle, SqliteError};
use crate::value::Value;

pub trait SqliteCallable: Send + Sync {
    unsafe fn call_boxed_closure(
        &self,
        ctx: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    );
    // number of arguments
    fn arg_count(&self) -> i32;
}

pub struct SqliteFunctionCtx {
    ctx: *mut sqlite3_context,
    argument_values: Vec<SqliteValue>,
}

impl SqliteFunctionCtx {
    /// Creates a new `SqliteFunctionCtx` from the given raw SQLite function context.
    /// The context is used to access the arguments passed to the function.
    /// Safety: the context must be valid and argc must be the number of arguments passed to the function.
    unsafe fn new(ctx: *mut sqlite3_context, argc: c_int, argv: *mut *mut sqlite3_value) -> Self {
        let count = usize::try_from(argc).expect("invalid argument count");
        let argument_values = (0..count)
            .map(|i| {
                let raw = *argv.add(i);
                let data_type_code = sqlite3_value_type(raw);
                let value_type_info = SqliteTypeInfo(DataType::from_code(data_type_code));
                SqliteValue::new(raw, value_type_info)
            })
            .collect::<Vec<_>>();
        Self {
            ctx,
            argument_values,
        }
    }

    /// Returns the argument at the given index, or panics if the argument number is out of bounds or
    /// the argument cannot be decoded as the requested type.
    pub fn get_arg<'q, T: Decode<'q, Sqlite>>(&'q self, index: usize) -> T {
        self.try_get_arg::<T>(index)
            .expect("invalid argument index")
    }

    /// Returns the argument at the given index, or `None` if the argument number is out of bounds or
    /// the argument cannot be decoded as the requested type.
    pub fn try_get_arg<'q, T: Decode<'q, Sqlite>>(&'q self, index: usize) -> Result<T, BoxDynError> {
        if let Some(value) = self.argument_values.get(index) {
            let value_ref = value.as_ref();
            T::decode(value_ref)
        } else {
            Err("invalid argument index".into())
        }
    }

    pub fn set_result<'q, R: Encode<'q, Sqlite>>(&self, result: R) {
        unsafe {
            let mut arg_buffer: Vec<SqliteArgumentValue<'q>> = Vec::with_capacity(1);
            if let IsNull::Yes = result.encode(&mut arg_buffer) {
                sqlite3_result_null(self.ctx);
            } else {
                let arg = arg_buffer.pop().unwrap();
                match arg {
                    SqliteArgumentValue::Null => {
                        sqlite3_result_null(self.ctx);
                    }
                    SqliteArgumentValue::Text(text) => {
                        sqlite3_result_text(
                            self.ctx,
                            text.as_ptr() as *const c_char,
                            text.len() as c_int,
                            SQLITE_TRANSIENT(),
                        );
                    }
                    SqliteArgumentValue::Blob(blob) => {
                        sqlite3_result_blob(
                            self.ctx,
                            blob.as_ptr() as *const c_void,
                            blob.len() as c_int,
                            SQLITE_TRANSIENT(),
                        );
                    }
                    SqliteArgumentValue::Double(double) => {
                        sqlite3_result_double(self.ctx, double);
                    }
                    SqliteArgumentValue::Int(int) => {
                        sqlite3_result_int(self.ctx, int);
                    }
                    SqliteArgumentValue::Int64(int64) => {
                        sqlite3_result_int64(self.ctx, int64);
                    }
                }
            }
        }
    }

    pub fn set_error(&self, error_str: &str) {
        let error_str = CString::new(error_str).expect("invalid error string");
        unsafe {
            sqlite3_result_error(
                self.ctx,
                error_str.as_ptr(),
                error_str.as_bytes().len() as c_int,
            );
        }
    }
}

impl<F: Fn(&SqliteFunctionCtx) + Send + Sync> SqliteCallable for F {
    unsafe fn call_boxed_closure(
        &self,
        ctx: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    ) {
        let ctx = SqliteFunctionCtx::new(ctx, argc, argv);
        (*self)(&ctx);
    }
    fn arg_count(&self) -> i32 {
        -1
    }
}

#[derive(Clone)]
pub struct Function {
    name: CString,
    func: Arc<dyn SqliteCallable>,
    /// the function always returns the same result given the same inputs
    pub deterministic: bool,
    /// the function may only be invoked from top-level SQL, and cannot be used in VIEWs or TRIGGERs nor in schema structures such as CHECK constraints, DEFAULT clauses, expression indexes, partial indexes, or generated columns.
    pub direct_only: bool,
    call:
        unsafe extern "C" fn(ctx: *mut sqlite3_context, argc: c_int, argv: *mut *mut sqlite3_value),
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("deterministic", &self.deterministic)
            .finish_non_exhaustive()
    }
}

impl Function {
    pub fn new<N, F>(name: N, func: F) -> Self
    where
        N: Into<Vec<u8>>,
        F: SqliteCallable + Send + Sync + 'static,
    {
        Function {
            name: CString::new(name).expect("invalid function name"),
            func: Arc::new(func),
            deterministic: false,
            direct_only: false,
            call: call_boxed_closure::<F>,
        }
    }

    pub(crate) fn create(&self, handle: &mut ConnectionHandle) -> Result<(), Error> {
        let raw_f = Arc::into_raw(Arc::clone(&self.func));
        let r = unsafe {
            sqlite3_create_function_v2(
                handle.as_ptr(),
                self.name.as_ptr(),
                self.func.arg_count(), // number of arguments
                self.sqlite_flags(),
                raw_f as *mut c_void,
                Some(self.call),
                None, // no step function for scalar functions
                None, // no final function for scalar functions
                None, // no need to free the function
            )
        };

        if r == SQLITE_OK {
            Ok(())
        } else {
            Err(Error::Database(Box::new(SqliteError::new(handle.as_ptr()))))
        }
    }

    fn sqlite_flags(&self) -> c_int {
        let mut flags = SQLITE_UTF8;
        if self.deterministic {
            flags |= SQLITE_DETERMINISTIC;
        }
        if self.direct_only {
            flags |= SQLITE_DIRECTONLY;
        }
        flags
    }

    pub fn deterministic(mut self) -> Self {
        self.deterministic = true;
        self
    }

    pub fn direct_only(mut self) -> Self {
        self.direct_only = true;
        self
    }
}

unsafe extern "C" fn call_boxed_closure<F: SqliteCallable>(
    ctx: *mut sqlite3_context,
    argc: c_int,
    argv: *mut *mut sqlite3_value,
) {
    let data = sqlite3_user_data(ctx);
    let boxed_f: *mut F = data as *mut F;
    debug_assert!(!boxed_f.is_null());
    let expected_argc = (*boxed_f).arg_count();
    debug_assert!(expected_argc == -1 || argc == expected_argc);
    (*boxed_f).call_boxed_closure(ctx, argc, argv);
}
