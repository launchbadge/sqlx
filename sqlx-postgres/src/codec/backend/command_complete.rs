use atoi::atoi;
use bytes::Bytes;
use memchr::memrchr;
use sqlx_core::{error::Error, io::Decode};

#[derive(Debug)]
pub(crate) struct CommandComplete {
    /// The command tag. This is usually a single word that identifies which SQL command
    /// was completed.
    tag: Bytes,
}

impl Decode<'_> for CommandComplete {
    #[inline]
    fn decode_with(buf: Bytes, _: ()) -> Result<Self, Error> {
        Ok(CommandComplete { tag: buf })
    }
}

impl CommandComplete {
    /// Returns the number of rows affected.
    /// If the command does not return rows (e.g., "CREATE TABLE"), returns 0.
    pub(crate) fn rows_affected(&self) -> u64 {
        // Look backwards for the first SPACE
        memrchr(b' ', &self.tag)
            // This is either a word or the number of rows affected
            .and_then(|i| atoi(&self.tag[(i + 1)..]))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_insert() {
        const DATA: &[u8] = b"INSERT 0 1214\0";

        let cc = CommandComplete::decode(Bytes::from_static(DATA)).unwrap();

        assert_eq!(cc.rows_affected(), 1214);
    }

    #[test]
    fn decode_begin() {
        const DATA: &[u8] = b"BEGIN\0";

        let cc = CommandComplete::decode(Bytes::from_static(DATA)).unwrap();

        assert_eq!(cc.rows_affected(), 0);
    }

    #[test]
    fn decode_update() {
        const DATA: &[u8] = b"UPDATE 5\0";

        let cc = CommandComplete::decode(Bytes::from_static(DATA)).unwrap();

        assert_eq!(cc.rows_affected(), 5);
    }
}

#[cfg(all(test, not(debug_assertions)))]
mod bench {
    #[bench]
    fn decode(b: &mut test::Bencher) {
        const DATA: &[u8] = b"INSERT 0 1214\0";

        b.iter(|| {
            let _ = CommandComplete::decode(test::black_box(Bytes::from_static(DATA)));
        });
    }

    #[bench]
    fn rows_affected(b: &mut test::Bencher) {
        const DATA: &[u8] = b"INSERT 0 1214\0";

        let data = CommandComplete::decode(Bytes::from_static(DATA)).unwrap();

        b.iter(|| {
            let _rows = test::black_box(&data).rows_affected();
        });
    }
}
