use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres},
    types::Type,
};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use core::{
    convert::TryFrom,
    ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};

bitflags::bitflags! {
  struct RangeFlags: u8 {
      const EMPTY = 0x01;
      const LB_INC = 0x02;
      const UB_INC = 0x04;
      const LB_INF = 0x08;
      const UB_INF = 0x10;
      const LB_NULL = 0x20;
      const UB_NULL = 0x40;
      const CONTAIN_EMPTY = 0x80;
  }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PgRange<T> {
    pub start: Bound<T>,
    pub end: Bound<T>,
}

impl<T> PgRange<T> {
    pub fn new(start: Bound<T>, end: Bound<T>) -> Self {
        Self { start, end }
    }
}

impl<'a, T> Decode<'a, Postgres> for PgRange<T>
where
    T: for<'b> Decode<'b, Postgres> + Type<Postgres> + 'a,
{
    fn accepts(ty: &PgTypeInfo) -> bool {
        [
            PgTypeInfo::INT4_RANGE,
            PgTypeInfo::NUM_RANGE,
            PgTypeInfo::TS_RANGE,
            PgTypeInfo::TSTZ_RANGE,
            PgTypeInfo::DATE_RANGE,
            PgTypeInfo::INT8_RANGE,
        ]
        .contains(ty)
    }

    fn decode(value: PgValueRef<'a>) -> Result<PgRange<T>, crate::error::BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                decode_binary(value.as_bytes()?, value.format, value.type_info)
            }
            PgValueFormat::Text => decode_str(value.as_str()?, value.format(), value.type_info),
        }
    }
}

impl<'a, T> Encode<'a, Postgres> for PgRange<T>
where
    T: for<'b> Encode<'b, Postgres> + 'a,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut flags = match self.start {
            Bound::Included(_) => RangeFlags::LB_INC,
            Bound::Excluded(_) => RangeFlags::empty(),
            Bound::Unbounded => RangeFlags::LB_INF,
        };

        flags |= match self.end {
            Bound::Included(_) => RangeFlags::UB_INC,
            Bound::Excluded(_) => RangeFlags::empty(),
            Bound::Unbounded => RangeFlags::UB_INF,
        };

        buf.write_u8(flags.bits()).unwrap();

        let mut write = |bound: &Bound<T>| -> IsNull {
            match bound {
                Bound::Included(ref value) | Bound::Excluded(ref value) => {
                    buf.write_u32::<NetworkEndian>(0).unwrap();
                    let prev = buf.len();
                    if let IsNull::Yes = Encode::<Postgres>::encode(value, buf) {
                        return IsNull::Yes;
                    }
                    let len = buf.len() - prev;
                    buf[prev - 4..prev].copy_from_slice(&(len as u32).to_be_bytes());
                }
                Bound::Unbounded => {}
            }
            IsNull::No
        };

        if let IsNull::Yes = write(&self.start) {
            return IsNull::Yes;
        }
        write(&self.end)
    }
}

impl<T> From<[Bound<T>; 2]> for PgRange<T> {
    fn from(from: [Bound<T>; 2]) -> Self {
        let [start, end] = from;
        Self { start, end }
    }
}

impl<T> From<(Bound<T>, Bound<T>)> for PgRange<T> {
    fn from(from: (Bound<T>, Bound<T>)) -> Self {
        Self {
            start: from.0,
            end: from.1,
        }
    }
}

impl<T> From<PgRange<T>> for [Bound<T>; 2] {
    fn from(from: PgRange<T>) -> Self {
        [from.start, from.end]
    }
}

impl<T> From<PgRange<T>> for (Bound<T>, Bound<T>) {
    fn from(from: PgRange<T>) -> Self {
        (from.start, from.end)
    }
}

impl<T> From<Range<T>> for PgRange<T> {
    fn from(from: Range<T>) -> Self {
        Self {
            start: Bound::Included(from.start),
            end: Bound::Excluded(from.end),
        }
    }
}

impl<T> From<RangeFrom<T>> for PgRange<T> {
    fn from(from: RangeFrom<T>) -> Self {
        Self {
            start: Bound::Included(from.start),
            end: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeInclusive<T>> for PgRange<T> {
    fn from(from: RangeInclusive<T>) -> Self {
        let (start, end) = from.into_inner();
        Self {
            start: Bound::Included(start),
            end: Bound::Excluded(end),
        }
    }
}

impl<T> From<RangeTo<T>> for PgRange<T> {
    fn from(from: RangeTo<T>) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Excluded(from.end),
        }
    }
}

impl<T> From<RangeToInclusive<T>> for PgRange<T> {
    fn from(from: RangeToInclusive<T>) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Included(from.end),
        }
    }
}

impl<T> RangeBounds<T> for PgRange<T> {
    fn start_bound(&self) -> Bound<&T> {
        match &self.start {
            Bound::Included(ref start) => Bound::Included(start),
            Bound::Excluded(ref start) => Bound::Excluded(start),
            Bound::Unbounded => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&T> {
        match &self.end {
            Bound::Included(ref end) => Bound::Included(end),
            Bound::Excluded(ref end) => Bound::Excluded(end),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

impl<T> TryFrom<PgRange<T>> for Range<T> {
    type Error = crate::error::Error;

    fn try_from(from: PgRange<T>) -> crate::error::Result<Self> {
        let err_msg = "Invalid data for core::ops::Range";
        let start = included(from.start, err_msg)?;
        let end = excluded(from.end, err_msg)?;
        Ok(start..end)
    }
}

impl<T> TryFrom<PgRange<T>> for RangeFrom<T> {
    type Error = crate::error::Error;

    fn try_from(from: PgRange<T>) -> crate::error::Result<Self> {
        let err_msg = "Invalid data for core::ops::RangeFrom";
        let start = included(from.start, err_msg)?;
        unbounded(from.end, err_msg)?;
        Ok(start..)
    }
}

impl<T> TryFrom<PgRange<T>> for RangeInclusive<T> {
    type Error = crate::error::Error;

    fn try_from(from: PgRange<T>) -> crate::error::Result<Self> {
        let err_msg = "Invalid data for core::ops::RangeInclusive";
        let start = included(from.start, err_msg)?;
        let end = included(from.end, err_msg)?;
        Ok(start..=end)
    }
}

impl<T> TryFrom<PgRange<T>> for RangeTo<T> {
    type Error = crate::error::Error;

    fn try_from(from: PgRange<T>) -> crate::error::Result<Self> {
        let err_msg = "Invalid data for core::ops::RangeTo";
        unbounded(from.start, err_msg)?;
        let end = excluded(from.end, err_msg)?;
        Ok(..end)
    }
}

impl<T> TryFrom<PgRange<T>> for RangeToInclusive<T> {
    type Error = crate::error::Error;

    fn try_from(from: PgRange<T>) -> crate::error::Result<Self> {
        let err_msg = "Invalid data for core::ops::RangeToInclusive";
        unbounded(from.start, err_msg)?;
        let end = included(from.end, err_msg)?;
        Ok(..=end)
    }
}

fn decode_binary<'r, T>(
    mut bytes: &[u8],
    format: PgValueFormat,
    type_info: PgTypeInfo,
) -> Result<PgRange<T>, crate::error::BoxDynError>
where
    T: for<'rec> Decode<'rec, Postgres> + 'r,
{
    let flags: RangeFlags = RangeFlags::from_bits_truncate(bytes.read_u8()?);
    let mut start_value = Bound::Unbounded;
    let mut end_value = Bound::Unbounded;

    if flags.contains(RangeFlags::EMPTY) {
        return Ok(PgRange {
            start: start_value,
            end: end_value,
        });
    }

    if !flags.contains(RangeFlags::LB_INF) {
        let elem_size = bytes.read_i32::<NetworkEndian>()?;
        let (elem_bytes, new_bytes) = bytes.split_at(elem_size as usize);
        bytes = new_bytes;
        let value = T::decode(PgValueRef {
            type_info: type_info.clone(),
            format,
            value: Some(elem_bytes),
            row: None,
        })?;

        start_value = if flags.contains(RangeFlags::LB_INC) {
            Bound::Included(value)
        } else {
            Bound::Excluded(value)
        };
    }

    if !flags.contains(RangeFlags::UB_INF) {
        bytes.read_i32::<NetworkEndian>()?;
        let value = T::decode(PgValueRef {
            type_info,
            format,
            value: Some(bytes),
            row: None,
        })?;

        end_value = if flags.contains(RangeFlags::UB_INC) {
            Bound::Included(value)
        } else {
            Bound::Excluded(value)
        };
    }

    Ok(PgRange {
        start: start_value,
        end: end_value,
    })
}

fn decode_str<'r, T>(
    s: &str,
    format: PgValueFormat,
    type_info: PgTypeInfo,
) -> Result<PgRange<T>, crate::error::BoxDynError>
where
    T: for<'rec> Decode<'rec, Postgres> + 'r,
{
    let err = || crate::error::Error::Decode("Invalid PostgreSQL range string".into());

    let value =
        |bound: &str, delim, bounds: [&str; 2]| -> Result<Bound<T>, crate::error::BoxDynError> {
            if bound.len() == 0 {
                return Ok(Bound::Unbounded);
            }
            let bound_value = T::decode(PgValueRef {
                type_info: type_info.clone(),
                format,
                value: Some(bound.as_bytes()),
                row: None,
            })?;
            if delim == bounds[0] {
                Ok(Bound::Excluded(bound_value))
            } else if delim == bounds[1] {
                Ok(Bound::Included(bound_value))
            } else {
                Err(Box::new(err()))
            }
        };

    let mut parts = s.split(',');
    let start_str = parts.next().ok_or_else(err)?;
    let start_value = value(
        start_str.get(1..).ok_or_else(err)?,
        start_str.get(0..1).ok_or_else(err)?,
        ["(", "["],
    )?;
    let end_str = parts.next().ok_or_else(err)?;
    let last_char_idx = end_str.len() - 1;
    let end_value = value(
        end_str.get(..last_char_idx).ok_or_else(err)?,
        end_str.get(last_char_idx..).ok_or_else(err)?,
        [")", "]"],
    )?;

    Ok(PgRange {
        start: start_value,
        end: end_value,
    })
}

fn excluded<T>(b: Bound<T>, err_msg: &str) -> crate::error::Result<T> {
    if let Bound::Excluded(rslt) = b {
        Ok(rslt)
    } else {
        Err(crate::error::Error::Decode(err_msg.into()))
    }
}

fn included<T>(b: Bound<T>, err_msg: &str) -> crate::error::Result<T> {
    if let Bound::Included(rslt) = b {
        Ok(rslt)
    } else {
        Err(crate::error::Error::Decode(err_msg.into()))
    }
}

fn unbounded<T>(b: Bound<T>, err_msg: &str) -> crate::error::Result<()> {
    if matches!(b, Bound::Unbounded) {
        Ok(())
    } else {
        Err(crate::error::Error::Decode(err_msg.into()))
    }
}
