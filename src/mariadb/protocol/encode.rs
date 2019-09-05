use super::Capabilities;

pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities);
}
