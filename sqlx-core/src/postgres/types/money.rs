use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres},
    types::Type,
};
use byteorder::{BigEndian, ByteOrder};
use std::{
    io,
    ops::{Add, AddAssign, Sub, SubAssign},
};

/// The PostgreSQL [`MONEY`] type stores a currency amount with a fixed fractional
/// precision. The fractional precision is determined by the database's
/// `lc_monetary` setting.
///
/// Data is read and written as 64-bit signed integers, and conversion into a
/// decimal should be done using the right precision.
///
/// Reading `MONEY` value in text format is not supported and will cause an error.
///
/// [`MONEY`]: https://www.postgresql.org/docs/current/datatype-money.html
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PgMoney(pub i64);

impl PgMoney {
    /// Convert the money value into a [`BigDecimal`] using the correct precision
    /// defined in the PostgreSQL settings. The default precision is two.
    ///
    /// [`BigDecimal`]: ../../types/struct.BigDecimal.html
    #[cfg(feature = "bigdecimal")]
    pub fn to_bigdecimal(self, scale: i64) -> bigdecimal::BigDecimal {
        let digits = num_bigint::BigInt::from(self.0);

        bigdecimal::BigDecimal::new(digits, scale)
    }

    /// Convert the money value into a [`Decimal`] using the correct precision
    /// defined in the PostgreSQL settings. The default precision is two.
    ///
    /// [`Decimal`]: ../../types/struct.BigDecimal.html
    #[cfg(feature = "decimal")]
    pub fn to_decimal(self, scale: u32) -> rust_decimal::Decimal {
        rust_decimal::Decimal::new(self.0, scale)
    }
}

impl Type<Postgres> for PgMoney {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::MONEY
    }
}

impl Type<Postgres> for [PgMoney] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::MONEY_ARRAY
    }
}

impl Type<Postgres> for Vec<PgMoney> {
    fn type_info() -> PgTypeInfo {
        <[PgMoney] as Type<Postgres>>::type_info()
    }
}

impl<T> From<T> for PgMoney
where
    T: Into<i64>,
{
    fn from(num: T) -> Self {
        Self(num.into())
    }
}

impl Encode<'_, Postgres> for PgMoney {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.0.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for PgMoney {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let cents = BigEndian::read_i64(value.as_bytes()?);

                Ok(PgMoney(cents))
            }
            PgValueFormat::Text => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Reading a `MONEY` value in text format is not supported.",
                );

                Err(Box::new(error))
            }
        }
    }
}

impl Add<PgMoney> for PgMoney {
    type Output = PgMoney;

    /// Adds two monetary values.
    ///
    /// # Panics
    /// Panics if overflowing the `i64::MAX`.
    fn add(self, rhs: PgMoney) -> Self::Output {
        self.0
            .checked_add(rhs.0)
            .map(PgMoney)
            .expect("overflow adding money amounts")
    }
}

impl AddAssign<PgMoney> for PgMoney {
    /// An assigning add for two monetary values.
    ///
    /// # Panics
    /// Panics if overflowing the `i64::MAX`.
    fn add_assign(&mut self, rhs: PgMoney) {
        self.0 = self
            .0
            .checked_add(rhs.0)
            .expect("overflow adding money amounts")
    }
}

impl Sub<PgMoney> for PgMoney {
    type Output = PgMoney;

    /// Subtracts two monetary values.
    ///
    /// # Panics
    /// Panics if underflowing the `i64::MIN`.
    fn sub(self, rhs: PgMoney) -> Self::Output {
        self.0
            .checked_sub(rhs.0)
            .map(PgMoney)
            .expect("overflow subtracting money amounts")
    }
}

impl SubAssign<PgMoney> for PgMoney {
    /// An assigning subtract for two monetary values.
    ///
    /// # Panics
    /// Panics if underflowing the `i64::MIN`.
    fn sub_assign(&mut self, rhs: PgMoney) {
        self.0 = self
            .0
            .checked_sub(rhs.0)
            .expect("overflow subtracting money amounts")
    }
}

#[cfg(test)]
mod tests {
    use super::PgMoney;

    #[test]
    fn adding_works() {
        assert_eq!(PgMoney(3), PgMoney(1) + PgMoney(2))
    }

    #[test]
    fn add_assign_works() {
        let mut money = PgMoney(1);
        money += PgMoney(2);

        assert_eq!(PgMoney(3), money);
    }

    #[test]
    fn subtracting_works() {
        assert_eq!(PgMoney(4), PgMoney(5) - PgMoney(1))
    }

    #[test]
    fn sub_assign_works() {
        let mut money = PgMoney(1);
        money -= PgMoney(2);

        assert_eq!(PgMoney(-1), money);
    }

    #[test]
    #[should_panic]
    fn add_overflow_panics() {
        let _ = PgMoney(i64::MAX) + PgMoney(1);
    }

    #[test]
    #[should_panic]
    fn add_assign_overflow_panics() {
        let mut money = PgMoney(i64::MAX);
        money += PgMoney(1);
    }

    #[test]
    #[should_panic]
    fn sub_overflow_panics() {
        let _ = PgMoney(i64::MIN) - PgMoney(1);
    }

    #[test]
    #[should_panic]
    fn sub_assign_overflow_panics() {
        let mut money = PgMoney(i64::MIN);
        money -= PgMoney(1);
    }

    #[test]
    #[cfg(feature = "bigdecimal")]
    fn conversion_to_bigdecimal_works() {
        let money = PgMoney(12345);

        assert_eq!(
            bigdecimal::BigDecimal::new(num_bigint::BigInt::from(12345), 2),
            money.to_bigdecimal(2)
        );
    }

    #[test]
    #[cfg(feature = "decimal")]
    fn conversion_to_decimal_works() {
        assert_eq!(
            rust_decimal::Decimal::new(12345, 2),
            PgMoney(12345).to_decimal(2)
        );
    }
}
