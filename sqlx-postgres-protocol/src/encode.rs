use std::io;

pub trait Encode {
    // TODO: Remove
    fn size_hint(&self) -> usize {
        0
    }

    // FIXME: Use BytesMut and not Vec<u8> (also remove the error type here)
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()>;
}
