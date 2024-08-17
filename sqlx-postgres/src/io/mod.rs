mod buf_mut;

pub use buf_mut::PgBufMutExt;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::{NonZeroU32, Saturating};

pub(crate) use sqlx_core::io::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct StatementId(IdInner);

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct PortalId(IdInner);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct IdInner(Option<NonZeroU32>);

impl StatementId {
    pub const UNNAMED: Self = Self(IdInner::UNNAMED);

    pub const NAMED_START: Self = Self(IdInner::NAMED_START);

    #[cfg(test)]
    pub const TEST_VAL: Self = Self(IdInner::TEST_VAL);

    const NAME_PREFIX: &'static str = "sqlx_s_";

    pub fn next(&self) -> Self {
        Self(self.0.next())
    }

    pub fn name_len(&self) -> Saturating<usize> {
        self.0.name_len(Self::NAME_PREFIX)
    }

    // There's no common trait implemented by `Formatter` and `Vec<u8>` for this purpose;
    // we're deliberately avoiding the formatting machinery because it's known to be slow.
    pub fn write_name<E>(&self, write: impl FnMut(&str) -> Result<(), E>) -> Result<(), E> {
        self.0.write_name(Self::NAME_PREFIX, write)
    }
}

impl Display for StatementId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.write_name(|s| f.write_str(s))
    }
}

#[allow(dead_code)]
impl PortalId {
    // None selects the unnamed portal
    pub const UNNAMED: Self = PortalId(IdInner::UNNAMED);

    pub const NAMED_START: Self = PortalId(IdInner::NAMED_START);

    #[cfg(test)]
    pub const TEST_VAL: Self = Self(IdInner::TEST_VAL);

    const NAME_PREFIX: &'static str = "sqlx_p_";

    /// If ID represents a named portal, return the next ID, wrapping on overflow.
    ///
    /// If this ID represents the unnamed portal, return the same.
    pub fn next(&self) -> Self {
        Self(self.0.next())
    }

    /// Calculate the number of bytes that will be written by [`Self::write_name()`].
    pub fn name_len(&self) -> Saturating<usize> {
        self.0.name_len(Self::NAME_PREFIX)
    }

    pub fn write_name<E>(&self, write: impl FnMut(&str) -> Result<(), E>) -> Result<(), E> {
        self.0.write_name(Self::NAME_PREFIX, write)
    }
}

impl IdInner {
    const UNNAMED: Self = Self(None);

    const NAMED_START: Self = Self(Some(NonZeroU32::MIN));

    #[cfg(test)]
    pub const TEST_VAL: Self = Self(NonZeroU32::new(1234567890));

    #[inline(always)]
    fn next(&self) -> Self {
        Self(
            self.0
                .map(|id| id.checked_add(1).unwrap_or(NonZeroU32::MIN)),
        )
    }

    #[inline(always)]
    fn name_len(&self, name_prefix: &str) -> Saturating<usize> {
        let mut len = Saturating(0);

        if let Some(id) = self.0 {
            len += name_prefix.len();
            // estimate the length of the ID in decimal
            // `.ilog10()` can't panic since the value is never zero
            len += id.get().ilog10() as usize;
            // add one to compensate for `ilog10()` rounding down.
            len += 1;
        }

        // count the NUL terminator
        len += 1;

        len
    }

    #[inline(always)]
    fn write_name<E>(
        &self,
        name_prefix: &str,
        mut write: impl FnMut(&str) -> Result<(), E>,
    ) -> Result<(), E> {
        if let Some(id) = self.0 {
            write(name_prefix)?;
            write(itoa::Buffer::new().format(id.get()))?;
        }

        write("\0")?;

        Ok(())
    }
}
