use bytes::Bytes;
use std::io;
use crate::{Encode,  Decode};

pub struct StartupMessage {
    version: u32,
    params: Bytes,
}
