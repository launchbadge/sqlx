use crate::io::Buf;
use crate::postgres::database::Postgres;
use byteorder::NetworkEndian;
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct NotificationResponse<'c> {
    pub(crate) process_id: u32,
    pub(crate) channel: Cow<'c, str>,
    pub(crate) payload: Cow<'c, str>,
}

impl<'c> NotificationResponse<'c> {
    pub(crate) fn read(mut buf: &'c [u8]) -> crate::Result<Postgres, Self> {
        let process_id = buf.get_u32::<NetworkEndian>()?;
        let channel = buf.get_str_nul()?;
        let payload = buf.get_str_nul()?;

        Ok(Self {
            process_id,
            channel: Cow::Borrowed(channel),
            payload: Cow::Borrowed(payload),
        })
    }

    pub(crate) fn into_owned(self) -> NotificationResponse<'static> {
        NotificationResponse {
            process_id: self.process_id,
            channel: Cow::Owned(self.channel.into_owned()),
            payload: Cow::Owned(self.payload.into_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NotificationResponse;

    const NOTIFICATION_RESPONSE: &[u8] = b"\x34\x20\x10\x02TEST-CHANNEL\0THIS IS A TEST\0";

    #[test]
    fn it_decodes_notification_response() {
        let message = NotificationResponse::read(NOTIFICATION_RESPONSE).unwrap();

        assert_eq!(message.process_id, 0x34201002);
        assert_eq!(&*message.channel, "TEST-CHANNEL");
        assert_eq!(&*message.payload, "THIS IS A TEST");
    }
}
