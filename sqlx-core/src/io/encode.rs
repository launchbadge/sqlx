pub trait Encode {
    // the buffer is guaranteed to be _empty_ when called
    fn encode(&self, buf: &mut Vec<u8>);
}
