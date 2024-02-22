use geo_types::{
    Error, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};
use geozero::wkb::{FromWkb, WkbDialect};
use geozero::{GeozeroGeometry, ToWkb};
use std::any::type_name;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::io::MySqlBufMutExt;
use crate::protocol::text::ColumnType;
use crate::types::Type;
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};

macro_rules! impl_mysql_type {
    ($name:ident) => {
        impl Type<MySql> for $name<f64> {
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
    };
}

impl_mysql_type!(Geometry);

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

/// Encode a subtype of [`Geometry`] into a MySQL value.
///
/// Override [`Encode::encode`] for each subtype to avoid the overhead of cloning the value.
macro_rules! impl_encode_subtype {
    ($name:ident) => {
        impl Encode<'_, MySql> for $name<f64> {
            fn encode(self, buf: &mut Vec<u8>) -> IsNull {
                Geometry::<f64>::$name(self).encode(buf)
            }

            fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
                Geometry::<f64>::$name(self.clone()).encode(buf)
            }
        }
    };
}

/// Decode a subtype of [`Geometry`] from a MySQL value.
///
/// All decodable geometry types in MySQL: `GEOMETRY`, `POINT`, `LINESTRING`, `POLYGON`, `MULTIPOINT`,
/// `MULTILINESTRING`, `MULTIPOLYGON`, `GEOMETRYCOLLECTION`.
///
/// [`Line`], [`Rect`], and [`Triangle`] can be encoded, but MySQL has no corresponding types.
/// This means, their [`TryFrom<Geometry<f64>>`] will always return [`Err`], so they are not decodable.
///
/// [`Line`]: geo_types::geometry::Line
/// [`Rect`]: geo_types::geometry::Rect
/// [`Triangle`]: geo_types::geometry::Triangle
macro_rules! impl_decode_subtype {
    ($name:ident) => {
        impl Decode<'_, MySql> for $name<f64> {
            fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
                Ok(<Geometry<f64> as Decode<'_, MySql>>::decode(value)?.try_into()?)
            }
        }
    };
}

macro_rules! impls_subtype {
    ($name:ident) => {
        impl_mysql_type!($name);
        impl_encode_subtype!($name);
        impl_decode_subtype!($name);
    };

    // GeometryCollection is a special case
    // Deprecated `GeometryCollection::from(single_geom)` produces unexpected results
    // TODO: remove it when GeometryCollection::from(single_geom) is removed
    ($name:ident, $n:ident => $($t:tt)+) => {
        impl_mysql_type!($name);
        impl_encode_subtype!($name);

        impl Decode<'_, MySql> for $name<f64> {
            fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
                let $n = <Geometry<f64> as Decode<'_, MySql>>::decode(value)?;

                $($t)+
            }
        }
    };
}

impls_subtype!(Point);
impls_subtype!(LineString);
impls_subtype!(Polygon);
impls_subtype!(MultiPoint);
impls_subtype!(MultiLineString);
impls_subtype!(MultiPolygon);

macro_rules! geometry_collection_mismatch {
    ($name:ident) => {
        Err(Error::MismatchedGeometry {
            expected: type_name::<GeometryCollection<f64>>(),
            found: type_name::<geo_types::geometry::$name<f64>>(),
        }
        .into())
    };
}

impls_subtype!(GeometryCollection, geom => match geom {
    Geometry::GeometryCollection(gc) => Ok(gc),
    Geometry::Point(_) => geometry_collection_mismatch!(Point),
    Geometry::Line(_) => geometry_collection_mismatch!(Line),
    Geometry::LineString(_) => geometry_collection_mismatch!(LineString),
    Geometry::Polygon(_) => geometry_collection_mismatch!(Polygon),
    Geometry::MultiPoint(_) => geometry_collection_mismatch!(MultiPoint),
    Geometry::MultiLineString(_) => geometry_collection_mismatch!(MultiLineString),
    Geometry::MultiPolygon(_) => geometry_collection_mismatch!(MultiPolygon),
    Geometry::Rect(_) => geometry_collection_mismatch!(Rect),
    Geometry::Triangle(_) => geometry_collection_mismatch!(Triangle),
});
