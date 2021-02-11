use crate::{Database, Runtime};

pub trait Decode<Rt: Runtime, Db: Database<Rt>>: Sized {
    fn decode(raw: &[u8]) -> crate::Result<Self>;
}
