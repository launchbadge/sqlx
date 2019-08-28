use std::{
    ascii::escape_default,
    fmt::{self, Debug},
    str::from_utf8,
};

// Wrapper type for byte slices that will debug print
// as a binary string
pub struct ByteStr<'a>(pub &'a [u8]);

impl Debug for ByteStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b\"")?;

        for &b in self.0 {
            let part: Vec<u8> = escape_default(b).collect();
            let s = from_utf8(&part).unwrap();

            write!(f, "{}", s)?;
        }

        write!(f, "\"")?;

        Ok(())
    }
}
