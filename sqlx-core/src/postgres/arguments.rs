use std::ops::{Deref, DerefMut};

use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::ext::ustr::UStr;
use crate::postgres::io::PgWriteExt;
use crate::postgres::{PgTypeInfo, Postgres};

/// Implementation of [`Arguments`] for PostgreSQL.
#[derive(Default)]
pub struct PgArguments {
    // Types of each bind parameter
    pub(crate) types: Vec<PgTypeInfo>,

    // Buffer of encoded bind parameters
    pub(crate) buffer: PgRawBuffer,
}

impl Arguments<'_> for PgArguments {
    type Database = Postgres;

    fn reserve(&mut self, additional: usize, size: usize) {
        self.types.reserve(additional);
        self.buffer.reserve(size);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<Self::Database>,
    {
        // remember the type information for this value
        self.types.push(T::produces());

        // reserve space to write the prefixed length of the value
        let offset = self.buffer.len();
        self.buffer.extend(&[0; 4]);

        // encode the value into our buffer
        let len = if let IsNull::No = value.encode(&mut self.buffer) {
            (self.buffer.len() - offset - 4) as i32
        } else {
            // Write a -1 to indicate NULL
            // NOTE: It is illegal for [encode] to write any data
            debug_assert_eq!(self.buffer.len(), offset + 4);
            -1_i32
        };

        // write the len to the beginning of the value
        self.buffer[offset..(offset + 4)].copy_from_slice(&len.to_be_bytes());
    }
}

/// A specialized buffer to encode bind arguments into for PostgreSQL.
#[derive(Default)]
pub struct PgRawBuffer {
    inner: Vec<u8>,

    // Whenever an `Encode` impl encounters a `PgTypeInfo` object that does not have an OID
    // It pushes a "hole" that must be patched later.
    //
    // The hole is a `usize` offset into the buffer with the type name that should be resolved
    // This is done for Records and Arrays as the OID is needed well before we are in an async
    // function and can just ask postgres.
    //
    type_holes: Vec<(usize, UStr)>, // Vec<{ offset, type_name }>
}

// TODO: PgRawBuffer#push_type_hole
// TODO: PgRawBuffer#patch_type_holes

impl Deref for PgRawBuffer {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PgRawBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
