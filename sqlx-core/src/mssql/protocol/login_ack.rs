use bytes::{Buf, Bytes};

use crate::error::Error;
use crate::mssql::io::MssqlBufExt;
use crate::mssql::protocol::pre_login::Version;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct LoginAck {
    pub(crate) interface: u8,
    pub(crate) tds_version: u32,
    pub(crate) program_name: String,
    pub(crate) program_version: Version,
}

impl LoginAck {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let mut data = buf.split_to(len as usize);

        let interface = data.get_u8();
        let tds_version = data.get_u32_le();
        let program_name = data.get_b_varchar()?;
        let program_version_major = data.get_u8();
        let program_version_minor = data.get_u8();
        let program_version_build = data.get_u16();

        Ok(Self {
            interface,
            tds_version,
            program_name,
            program_version: Version {
                major: program_version_major,
                minor: program_version_minor,
                build: program_version_build,
                sub_build: 0,
            },
        })
    }
}

#[test]
fn test_get() {
    #[rustfmt::skip]
    let mut buf = Bytes::from_static(&[
        0x36, 0, 1, 0x74, 0, 0, 4, 0x16, 0x4d, 0, 0x69, 0, 0x63, 0, 0x72, 0, 0x6f, 0, 0x73, 0, 0x6f, 0, 0x66, 0, 0x74, 0, 0x20, 0, 0x53, 0, 51, 0, 0x4c, 0, 0x20, 0, 0x53, 0, 0x65, 0, 0x72, 0, 0x76, 0, 0x65, 0, 0x72, 0, 0, 0, 0, 0, 0xf, 0, 0x10, 0x7f, 0xe3, 0x13, 0, 4, 4, 0x34, 0, 0x30, 0, 0x39, 0, 0x36, 0, 4, 0x34, 0, 0x30, 0, 0x39, 0, 0x36, 0, 0xfd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);

    let login_ack = LoginAck::get(&mut buf).unwrap();

    assert_eq!(login_ack.interface, 1);
    assert_eq!(login_ack.tds_version, 67108980);

    assert_eq!(login_ack.program_version.major, 15);
    assert_eq!(login_ack.program_version.minor, 0);
    assert_eq!(login_ack.program_version.build, 4223);
    assert_eq!(login_ack.program_version.sub_build, 0);

    assert_eq!(login_ack.program_name, "Microsoft S3L Server\0\0");
}
