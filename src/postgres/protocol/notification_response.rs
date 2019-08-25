use super::{Buf, Decode};
use byteorder::{BigEndian, ByteOrder};
use std::{fmt, io, pin::Pin, ptr::NonNull};

pub struct NotificationResponse {
    #[used]
    storage: Pin<Vec<u8>>,
    pid: u32,
    channel_name: NonNull<str>,
    message: NonNull<str>,
}

impl NotificationResponse {
    #[inline]
    pub fn pid(&self) -> u32 {
        self.pid
    }

    #[inline]
    pub fn channel_name(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.channel_name.as_ref() }
    }

    #[inline]
    pub fn message(&self) -> &str {
        // SAFE: Memory is pinned
        unsafe { self.message.as_ref() }
    }
}

// SAFE: Raw pointers point to pinned memory inside the struct
unsafe impl Send for NotificationResponse {}
unsafe impl Sync for NotificationResponse {}

impl fmt::Debug for NotificationResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NotificationResponse")
            .field("pid", &self.pid())
            .field("channel_name", &self.channel_name())
            .field("message", &self.message())
            .finish()
    }
}

impl Decode for NotificationResponse {
    fn decode(mut src: &[u8]) -> io::Result<Self> {
        let pid = src.get_u32()?;

        // offset from pid=4
        let storage = Pin::new(src.into());
        let mut src: &[u8] = &*storage;

        let channel_name = src.get_str_null()?.into();
        let message = src.get_str_null()?.into();

        Ok(Self {
            storage,
            pid,
            channel_name,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, NotificationResponse};
    use std::io;

    const NOTIFICATION_RESPONSE: &[u8] = b"\x34\x20\x10\x02TEST-CHANNEL\0THIS IS A TEST\0";

    #[test]
    fn it_decodes_notification_response() {
        let message = NotificationResponse::decode(NOTIFICATION_RESPONSE).unwrap();

        assert_eq!(message.pid(), 0x34201002);
        assert_eq!(message.channel_name(), "TEST-CHANNEL");
        assert_eq!(message.message(), "THIS IS A TEST");
    }
}
