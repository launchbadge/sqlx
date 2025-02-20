use crate::error::Error;
use std::ffi::CString;

use libsqlite3_sys::sqlite3_serialize;

use super::{ConnectionState, SqliteOwnedBuf};

/// Serializes the SQLite database into a byte vector.
///
/// This function returns the serialized bytes of the database for the specified `schema`
/// only if `deserialize` was previously called. If no data is available, it returns `None`.
///
/// # Arguments
/// * `schema` - The database schema to serialize (e.g., "main").
///
/// # Returns
/// * `Ok(Some(Vec<u8>))` - The serialized database as a byte vector.
/// * `Ok(None)` - No data is available to serialize.
/// * `Err(Error)` - An error occurred during serialization.
///
/// # See Also
/// [SQLite Documentation](https://sqlite.org/c3ref/serialize.html)
pub fn serialize(conn: &mut ConnectionState, schema: &str) -> Result<SqliteOwnedBuf, Error> {
    let mut size = 0;
    let c_schema = CString::new(schema).map_err(|e| Error::Configuration(Box::new(e)))?;

    let ptr = unsafe { sqlite3_serialize(conn.handle.as_ptr(), c_schema.as_ptr(), &mut size, 0) };

    let buf = SqliteOwnedBuf::from(ptr, usize::try_from(size).unwrap())
        .map_err(|e| Error::Configuration(Box::new(e)))?;

    Ok(buf)
}
