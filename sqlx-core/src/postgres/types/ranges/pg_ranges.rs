use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{
        types::ranges::pg_range::PgRange, PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres,
    },
    types::Type,
};

macro_rules! impl_pg_range {
    ($range_name:ident, $type_info:expr, $type_info_array:expr, $range_type:ty) => {
        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $range_name(pub PgRange<$range_type>);

        impl<'a> Decode<'a, Postgres> for $range_name {
            fn accepts(ty: &PgTypeInfo) -> bool {
                <PgRange<$range_type> as Decode<'_, Postgres>>::accepts(ty)
            }

            fn decode(value: PgValueRef<'a>) -> Result<$range_name, crate::error::BoxDynError> {
                Ok(Self(Decode::<Postgres>::decode(value)?))
            }
        }

        impl<'a> Encode<'a, Postgres> for $range_name {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                <PgRange<$range_type> as Encode<'_, Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        impl Type<Postgres> for $range_name {
            fn type_info() -> PgTypeInfo {
                $type_info
            }
        }

        impl Type<Postgres> for [$range_name] {
            fn type_info() -> PgTypeInfo {
                $type_info_array
            }
        }

        impl Type<Postgres> for Vec<$range_name> {
            fn type_info() -> PgTypeInfo {
                $type_info_array
            }
        }
    };
}

impl_pg_range!(
    Int4Range,
    PgTypeInfo::INT4_RANGE,
    PgTypeInfo::INT4_RANGE_ARRAY,
    i32
);
#[cfg(feature = "bigdecimal")]
impl_pg_range!(
    NumRange,
    PgTypeInfo::NUM_RANGE,
    PgTypeInfo::NUM_RANGE_ARRAY,
    bigdecimal::BigDecimal
);
#[cfg(feature = "chrono")]
impl_pg_range!(
    TsRange,
    PgTypeInfo::TS_RANGE,
    PgTypeInfo::TS_RANGE_ARRAY,
    chrono::NaiveDateTime
);
#[cfg(feature = "chrono")]
impl_pg_range!(
    DateRange,
    PgTypeInfo::DATE_RANGE,
    PgTypeInfo::DATE_RANGE_ARRAY,
    chrono::NaiveDate
);
impl_pg_range!(
    Int8Range,
    PgTypeInfo::INT8_RANGE,
    PgTypeInfo::INT8_RANGE_ARRAY,
    i64
);
