use std::cmp;
use std::convert::{TryFrom, TryInto};

use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::{PgTypeInfo, PgValue, Postgres};
use crate::types::Type;

use super::numeric::{PgNumeric, PgNumericSign};

impl Type<Postgres> for BigDecimal {
    fn type_info() -> PgTypeInfo {
        <PgNumeric as Type<Postgres>>::type_info()
    }
}

impl TryFrom<&'_ BigDecimal> for PgNumeric {
    type Error = std::num::TryFromIntError;

    fn try_from(bd: &'_ BigDecimal) -> Result<Self, Self::Error> {
        let base_10_to_10000 = |chunk: &[u8]| chunk.iter().fold(0i16, |a, &d| a * 10 + d as i16);

        // this implementation unfortunately has a number of redundant copies because BigDecimal
        // doesn't give us even immutable access to its internal representation, and neither
        // does `BigInt` or `BigUint`

        let (bigint, exp) = bd.as_bigint_and_exponent();
        // routine is specifically optimized for base-10
        let (sign, base_10) = bigint.to_radix_be(10);

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

impl TryFrom<PgNumeric> for BigDecimal {
    type Error = crate::Error;

    fn try_from(numeric: PgNumeric) -> crate::Result<Self> {
        let (digits, sign, weight) = match numeric {
            PgNumeric::Number {
                digits,
                sign,
                weight,
                ..
            } => (digits, sign, weight),
            PgNumeric::NotANumber => {
                return Err(crate::Error::Decode(
                    "BigDecimal does not support NaN values".into(),
                ))
            }
        };

        let sign = match sign {
            _ if digits.is_empty() => Sign::NoSign,
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
            .expect("BUG digit outside of given radix, check math above");

        Ok(BigDecimal::new(bigint, scale))
    }
}

/// ### Panics
/// If this `BigDecimal` cannot be represented by [PgNumeric].
impl Encode<Postgres> for BigDecimal {
    fn encode(&self, buf: &mut Vec<u8>) {
        PgNumeric::try_from(self)
            .expect("BigDecimal magnitude too great for Postgres NUMERIC type")
            .encode(buf);
    }

    fn size_hint(&self) -> usize {
        // BigDecimal::digits() gives us base-10 digits, so we divide by 4 to get base-10000 digits
        // and since this is just a hint we just always round up
        8 + (self.digits() / 4 + 1) as usize * 2
    }
}

impl Decode<'_, Postgres> for BigDecimal {
    fn decode(value: Option<PgValue>) -> crate::Result<Self> {
        match value.try_into()? {
            PgValue::Binary(binary) => PgNumeric::from_bytes(binary)?.try_into(),
            PgValue::Text(text) => text
                .parse::<BigDecimal>()
                .map_err(|e| crate::Error::Decode(e.into())),
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
