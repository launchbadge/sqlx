use super::decode::Decoder;
use failure::Error;

pub trait Deserialize: Sized {
    fn deserialize(decoder: &mut Decoder) -> Result<Self, Error>;
}
