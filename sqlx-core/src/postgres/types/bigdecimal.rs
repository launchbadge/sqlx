use std::cmp;
use std::convert::{TryFrom, TryInto};

use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::types::numeric::{PgNumeric, PgNumericSign};
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for BigDecimal {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::NUMERIC
    }
}

impl Type<Postgres> for [BigDecimal] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::NUMERIC_ARRAY
    }
}

impl Type<Postgres> for Vec<BigDecimal> {
    fn type_info() -> PgTypeInfo {
        <[BigDecimal] as Type<Postgres>>::type_info()
    }
}

impl TryFrom<PgNumeric> for BigDecimal {
    type Error = BoxDynError;

    fn try_from(numeric: PgNumeric) -> Result<Self, BoxDynError> {
        let (digits, sign, weight) = match numeric {
            PgNumeric::Number {
                digits,
                sign,
                weight,
                ..
            } => (digits, sign, weight),

            PgNumeric::NotANumber => {
                return Err("BigDecimal does not support NaN values".into());
            }
        };

        if digits.is_empty() {
            // Postgres returns an empty digit array for 0 but BigInt expects at least one zero
            return Ok(0u64.into());
        }

        let sign = match sign {
            PgNumericSign::Positive => Sign::Plus,
            PgNumericSign::Negative => Sign::Minus,
        };

        // weight is 0 if the decimal point falls after the first base-10000 digit
        let scale = (digits.len() as i64 - weight as i64 - 1) * 4;

        // no optimized algorithm for base-10 so use base-100 for faster processing
        let mut cents = Vec::with_capacity(digits.len() * 2);
        for digit in &digits {
            cents.push((digit / 100) as u8);
            cents.push((digit % 100) as u8);
        }

        let bigint = BigInt::from_radix_be(sign, &cents, 100)
            .ok_or("PgNumeric contained an out-of-range digit")?;

        Ok(BigDecimal::new(bigint, scale))
    }
}

impl TryFrom<&'_ BigDecimal> for PgNumeric {
    type Error = BoxDynError;

    fn try_from(decimal: &BigDecimal) -> Result<Self, BoxDynError> {
        let base_10_to_10000 = |chunk: &[u8]| chunk.iter().fold(0i16, |a, &d| a * 10 + d as i16);

        // NOTE: this unfortunately copies the BigInt internally
        let (integer, exp) = decimal.as_bigint_and_exponent();

        // this routine is specifically optimized for base-10
        // FIXME: is there a way to iterate over the digits to avoid the Vec allocation
        let (sign, base_10) = integer.to_radix_be(10);

        // weight is positive power of 10000
        // exp is the negative power of 10
        let weight_10 = base_10.len() as i64 - exp;

        // scale is only nonzero when we have fractional digits
        // since `exp` is the _negative_ decimal exponent, it tells us
        // exactly what our scale should be
        let scale: i16 = cmp::max(0, exp).try_into()?;

        // there's an implicit +1 offset in the interpretation
        let weight: i16 = if weight_10 <= 0 {
            weight_10 / 4 - 1
        } else {
            weight_10 / 4
        }
        .try_into()?;

        let digits_len = if base_10.len() % 4 != 0 {
            base_10.len() / 4 + 1
        } else {
            base_10.len() / 4
        };

        let offset = if weight_10 < 0 {
            4 - (-weight_10) % 4
        } else {
            weight_10 % 4
        } as usize;

        let mut digits = Vec::with_capacity(digits_len);

        if let Some(first) = base_10.get(..offset) {
            if offset != 0 {
                digits.push(base_10_to_10000(first));
            }
        }

        if let Some(rest) = base_10.get(offset..) {
            digits.extend(
                rest.chunks(4)
                    .map(|chunk| base_10_to_10000(chunk) * 10i16.pow(4 - chunk.len() as u32)),
            );
        }

        while let Some(&0) = digits.last() {
            digits.pop();
        }

        Ok(PgNumeric::Number {
            sign: match sign {
                Sign::Plus | Sign::NoSign => PgNumericSign::Positive,
                Sign::Minus => PgNumericSign::Negative,
            },
            scale,
            weight,
            digits,
        })
    }
}

/// ### Panics
/// If this `BigDecimal` cannot be represented by [PgNumeric].
impl Encode<'_, Postgres> for BigDecimal {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        PgNumeric::try_from(self)
            .expect("BigDecimal magnitude too great for Postgres NUMERIC type")
            .encode(buf);

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        // BigDecimal::digits() gives us base-10 digits, so we divide by 4 to get base-10000 digits
        // and since this is just a hint we just always round up
        8 + (self.digits() / 4 + 1) as usize * 2
    }
}

impl Decode<'_, Postgres> for BigDecimal {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => PgNumeric::decode(value.as_bytes()?)?.try_into(),
            PgValueFormat::Text => Ok(value.as_str()?.parse::<BigDecimal>()?),
        }
    }
}

#[test]
fn test_bigdecimal_to_pgnumeric() {
    let one: BigDecimal = "1".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&one).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 0,
            weight: 0,
            digits: vec![1]
        }
    );

    let ten: BigDecimal = "10".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&ten).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 0,
            weight: 0,
            digits: vec![10]
        }
    );

    let one_hundred: BigDecimal = "100".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&one_hundred).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 0,
            weight: 0,
            digits: vec![100]
        }
    );

    // BigDecimal doesn't normalize here
    let ten_thousand: BigDecimal = "10000".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&ten_thousand).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 0,
            weight: 1,
            digits: vec![1]
        }
    );

    let two_digits: BigDecimal = "12345".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&two_digits).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 0,
            weight: 1,
            digits: vec![1, 2345]
        }
    );

    let one_tenth: BigDecimal = "0.1".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&one_tenth).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 1,
            weight: -1,
            digits: vec![1000]
        }
    );

    let decimal: BigDecimal = "1.2345".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&decimal).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 4,
            weight: 0,
            digits: vec![1, 2345]
        }
    );

    let decimal: BigDecimal = "0.12345".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&decimal).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 5,
            weight: -1,
            digits: vec![1234, 5000]
        }
    );

    let decimal: BigDecimal = "0.01234".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&decimal).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 5,
            weight: -1,
            digits: vec![0123, 4000]
        }
    );

    let decimal: BigDecimal = "12345.67890".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&decimal).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 5,
            weight: 1,
            digits: vec![1, 2345, 6789]
        }
    );

    let one_digit_decimal: BigDecimal = "0.00001234".parse().unwrap();
    assert_eq!(
        PgNumeric::try_from(&one_digit_decimal).unwrap(),
        PgNumeric::Number {
            sign: PgNumericSign::Positive,
            scale: 8,
            weight: -2,
            digits: vec![1234]
        }
    );
}
