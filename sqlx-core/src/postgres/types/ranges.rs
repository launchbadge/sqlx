pub(crate) mod pg_range;
pub(crate) mod pg_ranges;

use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{types::PgRange, PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres},
    types::Type,
};
use core::{
    convert::TryInto,
    ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};

macro_rules! impl_range {
    ($range:ident) => {
        impl<'a, T> Decode<'a, Postgres> for $range<T>
        where
            T: for<'b> Decode<'b, Postgres> + Type<Postgres> + 'a,
        {
            fn accepts(ty: &PgTypeInfo) -> bool {
                <PgRange<T> as Decode<'_, Postgres>>::accepts(ty)
            }

            fn decode(value: PgValueRef<'a>) -> Result<$range<T>, crate::error::BoxDynError> {
                let bounds: PgRange<T> = Decode::<Postgres>::decode(value)?;
                let rslt = bounds.try_into()?;
                Ok(rslt)
            }
        }

        impl<'a, T> Encode<'a, Postgres> for $range<T>
        where
            T: Clone + for<'b> Encode<'b, Postgres> + 'a,
        {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                <PgRange<T> as Encode<'_, Postgres>>::encode(self.clone().into(), buf)
            }
        }
    };
}

impl_range!(Range);
impl_range!(RangeFrom);
impl_range!(RangeInclusive);
impl_range!(RangeTo);
impl_range!(RangeToInclusive);

#[test]
fn test_decode_str_bounds() {
    use crate::postgres::type_info::PgType;

    const EXC1: Bound<i32> = Bound::Excluded(1);
    const EXC2: Bound<i32> = Bound::Excluded(2);
    const INC1: Bound<i32> = Bound::Included(1);
    const INC2: Bound<i32> = Bound::Included(2);
    const UNB: Bound<i32> = Bound::Unbounded;

    let check = |s: &str, range_cmp: [Bound<i32>; 2]| {
        let pg_value = PgValueRef {
            type_info: PgTypeInfo(PgType::Int4Range),
            format: PgValueFormat::Text,
            value: Some(s.as_bytes()),
            row: None,
        };
        let range: PgRange<i32> = Decode::<Postgres>::decode(pg_value).unwrap();
        assert_eq!(Into::<[Bound<i32>; 2]>::into(range), range_cmp);
    };

    check("(,)", [UNB, UNB]);
    check("(,]", [UNB, UNB]);
    check("(,2)", [UNB, EXC2]);
    check("(,2]", [UNB, INC2]);
    check("(1,)", [EXC1, UNB]);
    check("(1,]", [EXC1, UNB]);
    check("(1,2)", [EXC1, EXC2]);
    check("(1,2]", [EXC1, INC2]);

    check("[,)", [UNB, UNB]);
    check("[,]", [UNB, UNB]);
    check("[,2)", [UNB, EXC2]);
    check("[,2]", [UNB, INC2]);
    check("[1,)", [INC1, UNB]);
    check("[1,]", [INC1, UNB]);
    check("[1,2)", [INC1, EXC2]);
    check("[1,2]", [INC1, INC2]);
}
