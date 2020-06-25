use std::ops::{Deref, DerefMut};

use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::postgres::{PgConnection, PgTypeInfo, Postgres};
use crate::types::Type;

#[derive(Default)]
pub struct PgArgumentBuffer {
    buffer: Vec<u8>,

    // Whenever an `Encode` impl encounters a `PgTypeInfo` object that does not have an OID
    // It pushes a "hole" that must be patched later.
    //
    // The hole is a `usize` offset into the buffer with the type name that should be resolved
    // This is done for Records and Arrays as the OID is needed well before we are in an async
    // function and can just ask postgres.
    //
    type_holes: Vec<(usize, UStr)>, // Vec<{ offset, type_name }>
}

/// Implementation of [`Arguments`] for PostgreSQL.
#[derive(Default)]
pub struct PgArguments {
    // Types of each bind parameter
    pub(crate) types: Vec<PgTypeInfo>,

    // Buffer of encoded bind parameters
    pub(crate) buffer: PgArgumentBuffer,
}

impl<'q> Arguments<'q> for PgArguments {
    type Database = Postgres;

    fn reserve(&mut self, additional: usize, size: usize) {
        self.types.reserve(additional);
        self.buffer.reserve(size);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<'q, Self::Database> + Type<Self::Database>,
    {
        // remember the type information for this value
        self.types
            .push(value.produces().unwrap_or_else(T::type_info));

        // encode the value into our buffer
        self.buffer.encode(value);
    }
}

impl PgArgumentBuffer {
    pub(crate) fn encode<'q, T>(&mut self, value: T)
    where
        T: Encode<'q, Postgres>,
    {
        // reserve space to write the prefixed length of the value
        let offset = self.len();
        self.extend(&[0; 4]);

        // encode the value into our buffer
        let len = if let IsNull::No = value.encode(self) {
            (self.len() - offset - 4) as i32
        } else {
            // Write a -1 to indicate NULL
            // NOTE: It is illegal for [encode] to write any data
            debug_assert_eq!(self.len(), offset + 4);
            -1_i32
        };

        // write the len to the beginning of the value
        self[offset..(offset + 4)].copy_from_slice(&len.to_be_bytes());
    }

    // Extends the inner buffer by enough space to have an OID
    // Remembers where the OID goes and type name for the OID
    pub(crate) fn push_type_hole(&mut self, type_name: &UStr) {
        let offset = self.len();

        self.extend_from_slice(&0_u32.to_be_bytes());
        self.type_holes.push((offset, type_name.clone()));
    }

    // Patch all remembered type holes
    // This should only go out and ask postgres if we have not seen the type name yet
    pub(crate) async fn patch_type_holes(&mut self, conn: &mut PgConnection) -> Result<(), Error> {
        for (offset, name) in &self.type_holes {
            let oid = conn.fetch_type_id_by_name(&*name).await?;
            self.buffer[*offset..(*offset + 4)].copy_from_slice(&oid.to_be_bytes());
        }

        Ok(())
    }
}

impl Deref for PgArgumentBuffer {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for PgArgumentBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
