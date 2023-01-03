use std::str::FromStr;

use bigdecimal::BigDecimal;
use ethereum_types::U256;
use num_bigint::Sign;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::types::numeric::PgNumeric;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

impl Type<Postgres> for U256 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::NUMERIC
    }
}

impl PgHasArrayType for U256 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::NUMERIC_ARRAY
    }
}

impl TryFrom<PgNumeric> for U256 {
    type Error = BoxDynError;

    fn try_from(numeric: PgNumeric) -> Result<Self, BoxDynError> {
        let bigdecimal =
            BigDecimal::try_from(numeric).expect("U256 failed to convert NUMERIC to BigDecimal");

        if bigdecimal.sign() == Sign::Minus {
            return Err("U256 is unsigned".into());
        }
        if bigdecimal.digits() > 0 {
            return Err("U256 doesn't support decimal digits".into());
        }

        let u256 =
            U256::from_dec_str(&bigdecimal.to_string()).expect("U256 failed from BigDecimal");

        Ok(u256)
    }
}

impl TryFrom<&'_ U256> for PgNumeric {
    type Error = BoxDynError;

    fn try_from(u256: &U256) -> Result<Self, BoxDynError> {
        let bigdecimal =
            BigDecimal::from_str(&u256.to_string()).expect("U256 failed to convert to BigDecimal");
        let numeric = PgNumeric::try_from(&bigdecimal)
            .expect("U256 failed to convert BigDecimal to PgNumeric");

        Ok(numeric)
    }
}

/// ### Panics
/// If this `U256` cannot be represented by `PgNumeric`.
impl Encode<'_, Postgres> for U256 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        PgNumeric::try_from(self)
            .expect("Failed to convert U256 to PgNumeric")
            .encode(buf);

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        // We use the same formula as the BigDecimal specific size_hint function.
        let bigdecimal =
            BigDecimal::from_str(&self.to_string()).expect("U256 failed to convert to BigDecimal");

        8 + (bigdecimal.digits() / 4 + 1) as usize * 2
    }
}

impl Decode<'_, Postgres> for U256 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => PgNumeric::decode(value.as_bytes()?)?.try_into(),
            PgValueFormat::Text => Ok(U256::from_dec_str(
                &value.as_str()?.parse::<BigDecimal>()?.to_string(),
            )
            .expect("U256 failed from BigDecimal")),
        }
    }
}
