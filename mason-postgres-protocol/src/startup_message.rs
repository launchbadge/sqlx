use crate::{Decode, Encode};
use bytes::Bytes;
use std::io;

pub struct StartupMessage {
    version: u32,
    params: Bytes,
}
