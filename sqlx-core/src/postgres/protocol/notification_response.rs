use crate::io::Buf;
use crate::postgres::protocol::Decode;
use byteorder::NetworkEndian;

#[derive(Debug)]
pub struct NotificationResponse {
    pub pid: u32,
    pub channel_name: String,
    pub message: String,
}

impl Decode for NotificationResponse {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let pid = buf.get_u32::<NetworkEndian>()?;
        let channel_name = buf.get_str_nul()?.to_owned();
        let message = buf.get_str_nul()?.to_owned();

        Ok(Self {
            pid,
            channel_name,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, NotificationResponse};

    const NOTIFICATION_RESPONSE: &[u8] = b"\x34\x20\x10\x02TEST-CHANNEL\0THIS IS A TEST\0";

    #[test]
    fn it_decodes_notification_response() {
        let message = NotificationResponse::decode(NOTIFICATION_RESPONSE).unwrap();

        assert_eq!(message.pid, 0x34201002);
        assert_eq!(message.channel_name, "TEST-CHANNEL");
        assert_eq!(message.message, "THIS IS A TEST");
    }
}
