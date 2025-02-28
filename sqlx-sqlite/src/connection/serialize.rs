use crate::{error::Error, SqliteError};
use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use libsqlite3_sys::{
    sqlite3_deserialize, sqlite3_free, sqlite3_malloc64, sqlite3_serialize,
    SQLITE_DESERIALIZE_FREEONCLOSE, SQLITE_DESERIALIZE_READONLY, SQLITE_DESERIALIZE_RESIZEABLE,
    SQLITE_OK,
};

use super::ConnectionState;

pub(crate) fn serialize(conn: &mut ConnectionState, schema: &str) -> Result<SqliteOwnedBuf, Error> {
    let mut size = 0;
    let c_schema = CString::new(schema).map_err(|e| Error::Configuration(Box::new(e)))?;

    let buf = unsafe {
        let ptr = sqlite3_serialize(conn.handle.as_ptr(), c_schema.as_ptr(), &mut size, 0);

        SqliteOwnedBuf::from(ptr, usize::try_from(size).unwrap())
            .map_err(|e| Error::Configuration(Box::new(e)))?
    };

    Ok(buf)
}

pub(crate) fn deserialize(
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

    unsafe {
        let rc = sqlite3_deserialize(
            conn.handle.as_ptr(),
            c_schema.as_ptr(),
            buf,
            i64::try_from(size).unwrap(),
            i64::try_from(size).unwrap(),
            flags,
        );

        if rc != SQLITE_OK {
            return Err(SqliteError::new(conn.handle.as_ptr()).into());
        }
    };

    Ok(())
}

/// Errors that could occurr when using `SqliteOwnedBuf`
#[derive(Debug, thiserror::Error)]
pub enum SqliteBufError {
    #[error("error initializing buffer using sqlite3_malloc")]
    Malloc,
}

/// Memory buffer owned and allocated by sqlite
#[derive(Debug)]
pub struct SqliteOwnedBuf {
    ptr: NonNull<u8>,
    size: usize,
}

unsafe impl Send for SqliteOwnedBuf {}
unsafe impl Sync for SqliteOwnedBuf {}

impl Drop for SqliteOwnedBuf {
    fn drop(&mut self) {
        unsafe {
            sqlite3_free(self.ptr.as_ptr().cast());
        }
    }
}

impl SqliteOwnedBuf {
    /// Uses `sqlite3_malloc` to allocate a buffer and returns a pointer to it
    fn with_capacity(size: usize) -> Result<Self, SqliteBufError> {
        unsafe {
            let ptr = sqlite3_malloc64(u64::try_from(size).unwrap()).cast::<u8>();
            Self::from(ptr, size)
        }
    }

    /// Creates a new mem buffer from a pointer that has been created with sqlite_malloc
    unsafe fn from(ptr: *mut u8, size: usize) -> Result<Self, SqliteBufError> {
        Ok(Self {
            ptr: NonNull::new(ptr).ok_or(SqliteBufError::Malloc)?,
            size,
        })
    }

    fn into_raw(self) -> (*mut u8, usize) {
        let raw = (self.ptr.as_ptr(), self.size);
        // this is used in sqlite_deserialize and
        // underlying buffer must not be freed
        std::mem::forget(self);
        raw
    }
}

impl TryFrom<&[u8]> for SqliteOwnedBuf {
    type Error = Box<SqliteBufError>;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Self::with_capacity(bytes.len())?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf.ptr.as_mut(), buf.size);
        }

        Ok(buf)
    }
}

impl Deref for SqliteOwnedBuf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }
}

impl DerefMut for SqliteOwnedBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_mut(), self.size) }
    }
}

impl AsRef<[u8]> for SqliteOwnedBuf {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl AsMut<[u8]> for SqliteOwnedBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        self.deref_mut()
    }
}
