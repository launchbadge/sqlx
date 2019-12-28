use crate::io::Buf;
use crate::postgres::protocol::Decode;

#[derive(Debug)]
pub struct ParameterStatus {
    pub name: Box<str>,
    pub value: Box<str>,
}

impl Decode for ParameterStatus {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let name = buf.get_str_nul()?.into();
        let value = buf.get_str_nul()?.into();

        Ok(Self { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ParameterStatus};

    const PARAM_STATUS: &[u8] = b"session_authorization\0postgres\0";

    #[test]
    fn it_decodes_param_status() {
        let message = ParameterStatus::decode(PARAM_STATUS).unwrap();

        assert_eq!(&*message.name, "session_authorization");
        assert_eq!(&*message.value, "postgres");
    }
}
