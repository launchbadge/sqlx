use smallvec::SmallVec;
use sqlx_core::bytes::{Buf, Bytes};

use crate::error::Error;
use crate::io::Decode;
use crate::types::Oid;

#[derive(Debug)]
pub struct ParameterDescription {
    pub types: SmallVec<[Oid; 6]>,
}

impl Decode<'_> for ParameterDescription {
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self, Error> {
        let cnt = buf.get_u16();
        let mut types = SmallVec::with_capacity(cnt as usize);

        for _ in 0..cnt {
            types.push(Oid(buf.get_u32()));
        }

        Ok(Self { types })
    }
}

#[test]
fn test_decode_parameter_description() {
    const DATA: &[u8] = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";

    let m = ParameterDescription::decode(DATA.into()).unwrap();

    assert_eq!(m.types.len(), 2);
    assert_eq!(m.types[0], Oid(0x0000_0000));
    assert_eq!(m.types[1], Oid(0x0000_0500));
}

#[test]
fn test_decode_empty_parameter_description() {
    const DATA: &[u8] = b"\x00\x00";

    let m = ParameterDescription::decode(DATA.into()).unwrap();

    assert!(m.types.is_empty());
}

#[cfg(all(test, not(debug_assertions)))]
#[bench]
fn bench_decode_parameter_description(b: &mut test::Bencher) {
    const DATA: &[u8] = b"\x00\x02\x00\x00\x00\x00\x00\x00\x05\x00";

    b.iter(|| {
        ParameterDescription::decode(test::black_box(Bytes::from_static(DATA))).unwrap();
    });
}
