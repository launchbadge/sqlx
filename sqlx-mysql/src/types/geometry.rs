use geo_types::Geometry;
use geozero::wkb::{FromWkb, WkbDialect};
use geozero::{GeozeroGeometry, ToWkb};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::io::MySqlBufMutExt;
use crate::protocol::text::ColumnType;
use crate::types::Type;
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};

/// Define a type that can be used to represent a `GEOMETRY` field.
///
/// Note: Only `Geometry<f64>` is implemented with geozero::GeozeroGeometry.
impl Type<MySql> for Geometry<f64> {
    fn type_info() -> MySqlTypeInfo {
        // MySQL does not allow to execute with a Geometry parameter for now.
        // MySQL reports: 1210 (HY000): Incorrect arguments to mysqld_stmt_execute
        // MariaDB does not report errors but does not work properly.
        // So we use the `Blob` type to pass Geometry parameters.
        MySqlTypeInfo::binary(ColumnType::Blob)
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        ty.r#type == ColumnType::Geometry || <&[u8] as Type<MySql>>::compatible(ty)
    }
}

const ENCODE_ERR: &str = "failed to encode value as Geometry to WKB; the most likely cause is that the value is not a valid geometry";

impl Encode<'_, MySql> for Geometry<f64> {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        // Encoding is supposed to be infallible, so we don't have much choice but to panic here.
        // However, in most cases, a geometry being unable to serialize to WKB is most likely due to user error.
        let bytes = self.to_mysql_wkb(self.srid()).expect(ENCODE_ERR);

        buf.put_bytes_lenenc(bytes.as_ref());

        IsNull::No
    }
}

impl Decode<'_, MySql> for Geometry<f64> {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let mut bytes = value.as_bytes()?;

        Ok(FromWkb::from_wkb(&mut bytes, WkbDialect::MySQL)?)
    }
}
