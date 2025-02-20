use std::ffi::CString;

use libsqlite3_sys::{
    sqlite3_deserialize, SQLITE_DESERIALIZE_FREEONCLOSE, SQLITE_DESERIALIZE_READONLY,
    SQLITE_DESERIALIZE_RESIZEABLE, SQLITE_OK,
};

use crate::{error::Error, SqliteError};

use super::{ConnectionState, SqliteOwnedBuf};

/// Deserializes a SQLite database from a byte array into the specified schema.
///
/// This function uses SQLite's `sqlite3_deserialize` to load a database from a byte array
/// into the given schema (e.g., "main"). The memory for the byte array is managed by SQLite
/// and will be freed when the database is closed or the schema is reset.
///
/// # Safety
/// The memory for the `data` byte array is allocated using `sqlite3_malloc` and will be
/// freed by SQLite when the database is closed or the schema is reset. Do not use the
/// `data` byte array after calling this function.
///
/// # Notes
/// - The `SQLITE_DESERIALIZE_FREEONCLOSE` flag is used, so SQLite will automatically free
///   the memory when the database is closed or the schema is reset.
/// - If `read_only` is `true`, the `SQLITE_DESERIALIZE_READONLY` flag is also set, preventing
///   modifications to the deserialized database.
///
/// # See Also
/// [SQLite Documentation](https://sqlite.org/c3ref/deserialize.html)
pub fn deserialize(
    conn: &mut ConnectionState,
    schema: &str,
    data: &[u8],
    read_only: bool,
) -> Result<(), Error> {
    let c_schema = CString::new(schema).map_err(|e| Error::Configuration(Box::new(e)))?;

    // always use freeonclose flag here as the buffer
    // is allocated with sqlite3_malloc and therefore owned by sqlite
    let mut flags = SQLITE_DESERIALIZE_FREEONCLOSE;
    if read_only {
        flags |= SQLITE_DESERIALIZE_READONLY;
    } else {
        flags |= SQLITE_DESERIALIZE_RESIZEABLE;
    }

    let sqlite_buf = SqliteOwnedBuf::try_from(data).map_err(|e| Error::Configuration(e))?;

    // this function prevents the buf to be dropped which is okay here because
    // we're using SQLITE_DESERIALIZE_FREEONCLOSE which delegates sqlite to freeing memory
    // when db connection is closed
    let (buf, size) = sqlite_buf.into_raw();

    let rc = unsafe {
        sqlite3_deserialize(
            conn.handle.as_ptr(),
            c_schema.as_ptr(),
            buf,
            i64::try_from(size).unwrap(),
            i64::try_from(size).unwrap(),
            flags,
        )
    };

    if rc != SQLITE_OK {
        return Err(SqliteError::new(conn.handle.as_ptr()).into());
    }

    Ok(())
}
