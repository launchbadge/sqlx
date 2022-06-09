use bytes::{Buf, Bytes};

use crate::error::Error;
use crate::mssql::io::MssqlBufExt;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Info {
    pub(crate) number: u32,
    pub(crate) state: u8,
    pub(crate) class: u8,
    pub(crate) message: String,
    pub(crate) server: String,
    pub(crate) procedure: String,
    pub(crate) line: u32,
}

impl Info {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let mut data = buf.split_to(len as usize);

        let number = data.get_u32_le();
        let state = data.get_u8();
        let class = data.get_u8();
        let message = data.get_us_varchar()?;
        let server = data.get_b_varchar()?;
        let procedure = data.get_b_varchar()?;
        let line = data.get_u32_le();

        Ok(Self {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        })
    }
}

#[test]
fn test_get() {
    #[rustfmt::skip]
    let mut buf = Bytes::from_static(&[
        0x74, 0, 0x47, 0x16, 0, 0, 1, 0, 0x27, 0, 0x43, 0, 0x68, 0, 0x61, 0, 0x6e, 0, 0x67, 0, 0x65, 0, 0x64, 0, 0x20, 0, 0x6c, 0, 0x61, 0, 0x6e, 0, 0x67, 0, 0x75, 0, 0x61, 0, 0x67, 0, 0x65, 0, 0x20, 0, 0x73, 0, 0x65, 0, 0x74, 0, 0x74, 0, 0x69, 0, 0x6e, 0, 0x67, 0, 0x20, 0, 0x74, 0, 0x6f, 0, 0x20, 0, 0x75, 0, 0x73, 0, 0x5f, 0, 0x65, 0, 0x6e, 0, 0x67, 0, 0x6c, 0, 0x69, 0, 0x73, 0, 0x68, 0, 0x2e, 0, 0xc, 0x61, 0, 0x62, 0, 0x64, 0, 0x30, 0, 0x62, 0, 0x36, 0, 0x37, 0, 0x62, 0, 0x64, 0, 0x34, 0, 0x39, 0, 0x33, 0, 0, 1, 0, 0, 0, 0xad, 0x36, 0, 1, 0x74, 0, 0, 4, 0x16, 0x4d, 0, 0x69, 0, 0x63, 0, 0x72, 0, 0x6f, 0, 0x73, 0, 0x6f, 0, 0x66, 0, 0x74, 0, 0x20, 0, 0x53, 0, 0x51, 0, 0x4c, 0, 0x20, 0, 0x53, 0, 0x65, 0, 0x72, 0, 0x76, 0, 0x65, 0, 0x72, 0, 0, 0, 0, 0, 0xf, 0, 0x10, 0x7f, 0xe3, 0x13, 0, 4, 4, 0x34, 0, 0x30, 0, 0x39, 0, 0x36, 0, 4, 0x34, 0, 0x30, 0, 0x39, 0, 0x36, 0, 0xfd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);

    let info = Info::get(&mut buf).unwrap();

    assert_eq!(info.number, 5703);
    assert_eq!(info.state, 1);
    assert_eq!(info.class, 0);
    assert_eq!(info.message, "Changed language setting to us_english.");
    assert_eq!(info.server, "abd0b67bd493");
    assert_eq!(info.procedure, "");
    assert_eq!(info.line, 1);
}
