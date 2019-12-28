use chrono::{NaiveDateTime, Timelike};

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::MySql;

impl Encode<MySql> for NaiveDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        unimplemented!()
    }

    fn size_hint(&self) -> usize {
        match (
            self.hour(),
            self.minute(),
            self.second(),
            self.timestamp_subsec_micros(),
        ) {
            // include the length byte
            (0, 0, 0, 0) => 5,
            (_, _, _, 0) => 8,
            (_, _, _, _) => 12,
        }
    }
}

impl Decode<MySql> for NaiveDateTime {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        unimplemented!()
    }
}
