use crate::error::Error;
use bytes::{Buf, Bytes};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    // Fixed-length data types
    // https://docs.microsoft.com/en-us/openspecs/sql_server_protocols/ms-sstds/d33ef17b-7e53-4380-ad11-2ba42c8dda8d
    Null = 0x1f,
    TinyInt = 0x30,
    Bit = 0x32,
    SmallInt = 0x34,
    Int = 0x38,
    SmallDateTime = 0x3a,
    Real = 0x3b,
    Money = 0x3c,
    DateTime = 0x3d,
    Float = 0x3e,
    SmallMoney = 0x7a,
    BigInt = 0x7f,
}

// http://msdn.microsoft.com/en-us/library/dd358284.aspx
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeInfo {
    pub(crate) ty: DataType,
}

impl TypeInfo {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let ty = DataType::get(buf)?;

        Ok(Self { ty })
    }

    pub(crate) fn is_null(&self) -> bool {
        matches!(self.ty, DataType::Null)
    }

    pub(crate) fn size(&self) -> usize {
        match self.ty {
            DataType::Null => 0,
            DataType::TinyInt => 1,
            DataType::Bit => 1,
            DataType::SmallInt => 2,
            DataType::Int => 4,
            DataType::SmallDateTime => 4,
            DataType::Real => 4,
            DataType::Money => 4,
            DataType::DateTime => 8,
            DataType::Float => 8,
            DataType::SmallMoney => 4,
            DataType::BigInt => 8,
        }
    }
}

impl DataType {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(match buf.get_u8() {
            0x1f => DataType::Null,
            0x30 => DataType::TinyInt,
            0x32 => DataType::Bit,
            0x34 => DataType::SmallInt,
            0x38 => DataType::Int,
            0x3a => DataType::SmallDateTime,
            0x3b => DataType::Real,
            0x3c => DataType::Money,
            0x3d => DataType::DateTime,
            0x3e => DataType::Float,
            0x7a => DataType::SmallMoney,
            0x7f => DataType::BigInt,

            ty => {
                return Err(err_protocol!("unknown data type 0x{:02x}", ty));
            }
        })
    }
}
