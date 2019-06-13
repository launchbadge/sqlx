pub trait Serialize {
    fn serialize(&self, buf: &mut Vec<u8>);
}

#[derive(Debug)]
pub struct StartupMessage<'a> {
    pub host: &'a str,
}

impl<'a> Serialize for StartupMessage<'a> {
    fn serialize(&self, buf: &mut Vec<u8>) {
    }
}
