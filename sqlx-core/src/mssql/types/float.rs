use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for f32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Real | DataType::FloatN) && ty.0.size == 4
    }
}

impl Encode<'_, Mssql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_f32(value.as_bytes()?))
    }
}

impl Type<Mssql> for f64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::Float
                | DataType::FloatN
                | DataType::Decimal
                | DataType::DecimalN
                | DataType::Numeric
                | DataType::NumericN
        )
    }
}

impl Encode<'_, Mssql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.0.ty;
        let size = value.type_info.0.size;
        let precision = value.type_info.0.precision;
        let scale = value.type_info.0.scale;
        match ty {
            DataType::Float | DataType::FloatN if size == 8 => {
                Ok(LittleEndian::read_f64(value.as_bytes()?))
            }
            DataType::Numeric | DataType::NumericN | DataType::Decimal | DataType::DecimalN => {
                decode_numeric(value.as_bytes()?, precision, scale)
            }
            _ => Err(err_protocol!(
                "Decoding {:?} as a float failed because type {:?} is not implemented",
                value,
                ty
            )
            .into()),
        }
    }
}
fn decode_numeric(bytes: &[u8], _precision: u8, mut scale: u8) -> Result<f64, BoxDynError> {
    let sign = if bytes[0] == 0 { -1. } else { 1. };
    let rest = &bytes[1..];
    let mut fixed_bytes = [0u8; 16];
    fixed_bytes[0..rest.len()].copy_from_slice(rest);
    let mut numerator = u128::from_le_bytes(fixed_bytes);
    let mut decimal_part = 0f64;
    while scale > 0 {
        scale -= 1;
        decimal_part += (numerator % 10) as f64;
        decimal_part /= 10.;
        numerator /= 10;
    }
    Ok(sign * ((numerator as f64) + decimal_part))
}
