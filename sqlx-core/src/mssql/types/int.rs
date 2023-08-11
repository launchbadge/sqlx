use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for i8 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 1))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::TinyInt | DataType::IntN) && ty.0.size == 1
    }
}

impl Encode<'_, Mssql> for i8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value.as_bytes()?[0] as i8)
    }
}

impl Type<Mssql> for i16 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 2))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::SmallInt | DataType::IntN) && ty.0.size == 2
    }
}

impl Encode<'_, Mssql> for i16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i16(value.as_bytes()?))
    }
}

impl Type<Mssql> for i32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Int | DataType::IntN) && ty.0.size == 4
    }
}

impl Encode<'_, Mssql> for i32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i32(value.as_bytes()?))
    }
}

impl Type<Mssql> for i64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::SmallInt
                | DataType::Int
                | DataType::TinyInt
                | DataType::BigInt
                | DataType::IntN
                | DataType::Numeric
                | DataType::NumericN
                | DataType::Decimal
                | DataType::DecimalN
        )
    }
}

impl Encode<'_, Mssql> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.0.ty;
        let precision = value.type_info.0.precision;
        let scale = value.type_info.0.scale;
        match ty {
            DataType::SmallInt
            | DataType::Int
            | DataType::TinyInt
            | DataType::BigInt
            | DataType::IntN => {
                let mut buf = [0u8; 8];
                let bytes_val = value.as_bytes()?;
                buf[..bytes_val.len()].copy_from_slice(bytes_val);
                Ok(i64::from_le_bytes(buf))
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

fn decode_numeric(bytes: &[u8], _precision: u8, mut scale: u8) -> Result<i64, BoxDynError> {
    let negative = bytes[0] == 0;
    let rest = &bytes[1..];
    let mut fixed_bytes = [0u8; 16];
    fixed_bytes[0..rest.len()].copy_from_slice(rest);
    let mut numerator = u128::from_le_bytes(fixed_bytes);
    while scale > 0 {
        scale -= 1;
        numerator /= 10;
    }
    let n = i64::try_from(numerator)?;
    Ok(n * if negative { -1 } else { 1 })
}
