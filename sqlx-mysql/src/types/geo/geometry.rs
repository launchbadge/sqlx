use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

use crate::io::MySqlBufMutExt;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};

use geo_traits::to_geo::ToGeoGeometry;
use geo_types::Geometry;
use wkb::reader;
use wkb::writer;

impl Type<MySql> for Geometry {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Geometry)
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::Geometry
                | ColumnType::Blob
                | ColumnType::MediumBlob
                | ColumnType::LongBlob
                | ColumnType::TinyBlob
                | ColumnType::VarString if ty.flags.contains(ColumnFlags::BINARY)
        )
    }
}

impl Encode<'_, MySql> for Geometry {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        let mut wkb_buffer = Vec::new();
        wkb_buffer.extend_from_slice(&0u32.to_le_bytes()); // SRID = 0, Little Endian
        writer::write_geometry(&mut wkb_buffer, self, &writer::WriteOptions::default())?;
        buf.put_bytes_lenenc(&wkb_buffer);
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, MySql> for Geometry {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        if bytes.len() < 4 {
            return Err(format!(
                "Invalid GEOMETRY data for Geometry: received {} bytes, expected at least 4 for SRID prefix.",
                bytes.len()
            )
            .into());
        }
        let wkb_data = &bytes[4..]; // Skip 4-byte SRID

        let wkb_reader_geom = reader::Wkb::try_new(wkb_data)
            .map_err(|e| BoxDynError::from(format!("WKB parsing error for Geometry: {}", e)))?;

        Ok(wkb_reader_geom.to_geometry())
    }
}

#[cfg(test)]
mod tests {
    use geo_types::{coord, Geometry as TestableGeoType, LineString, Point, Polygon};
    use sqlx::mysql::{MySqlPool, MySqlRow};
    use sqlx::{Executor, Row};

    #[sqlx::test]
    async fn test_encode_decode_geometry_enum(pool: MySqlPool) -> anyhow::Result<()> {
        let table_name = format!("test_geo_geometry_enum_table");
        pool.execute(format!("DROP TABLE IF EXISTS {}", table_name).as_str())
            .await?;
        pool.execute(
            format!(
                "CREATE TABLE {} (id INT, geom_val GEOMETRY, geom_null GEOMETRY NULL)",
                table_name
            )
            .as_str(),
        )
        .await?;

        let p1_geom = Point::new(1.0, 2.0);
        let geom_enum1 = TestableGeoType::Point(p1_geom);

        let ls1_geom = LineString::new(vec![coord! {x: 3., y: 4.}, coord! {x: 5., y: 6.}]);
        let geom_enum2 = TestableGeoType::LineString(ls1_geom);

        let ext_poly_geom = LineString::new(vec![
            coord! {x:0.,y:0.},
            coord! {x:1.,y:1.},
            coord! {x:1.,y:0.},
            coord! {x:0.,y:0.},
        ]);
        let poly1_geom = Polygon::new(ext_poly_geom, vec![]);
        let geom_enum3 = TestableGeoType::Polygon(poly1_geom);

        // Test non-nullable Point variant
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_val) VALUES (1, ?)",
            table_name
        ))
        .bind(geom_enum1.clone())
        .execute(&pool)
        .await?;
        let row1: MySqlRow =
            sqlx::query(&format!("SELECT geom_val FROM {} WHERE id = 1", table_name))
                .fetch_one(&pool)
                .await?;
        let decoded_val1: TestableGeoType = row1.try_get("geom_val")?;
        assert_eq!(decoded_val1, geom_enum1);

        // Test non-nullable LineString variant
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_val) VALUES (2, ?)",
            table_name
        ))
        .bind(geom_enum2.clone())
        .execute(&pool)
        .await?;
        let row2: MySqlRow =
            sqlx::query(&format!("SELECT geom_val FROM {} WHERE id = 2", table_name))
                .fetch_one(&pool)
                .await?;
        let decoded_val2: TestableGeoType = row2.try_get("geom_val")?;
        assert_eq!(decoded_val2, geom_enum2);

        // Test non-nullable Polygon variant
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_val) VALUES (3, ?)",
            table_name
        ))
        .bind(geom_enum3.clone())
        .execute(&pool)
        .await?;
        let row3: MySqlRow =
            sqlx::query(&format!("SELECT geom_val FROM {} WHERE id = 3", table_name))
                .fetch_one(&pool)
                .await?;
        let decoded_val3: TestableGeoType = row3.try_get("geom_val")?;
        assert_eq!(decoded_val3, geom_enum3);

        // Test nullable Some(value) - using Point variant
        let some_val: Option<TestableGeoType> = Some(geom_enum1.clone());
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_null) VALUES (4, ?)",
            table_name
        ))
        .bind(some_val.clone())
        .execute(&pool)
        .await?;
        let row_some: MySqlRow = sqlx::query(&format!(
            "SELECT geom_null FROM {} WHERE id = 4",
            table_name
        ))
        .fetch_one(&pool)
        .await?;
        let decoded_some: Option<TestableGeoType> = row_some.try_get("geom_null")?;
        assert_eq!(decoded_some, some_val);

        // Test nullable None
        let none_val: Option<TestableGeoType> = None;
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_null) VALUES (5, ?)",
            table_name
        ))
        .bind(none_val.clone())
        .execute(&pool)
        .await?;
        let row_none: MySqlRow = sqlx::query(&format!(
            "SELECT geom_null FROM {} WHERE id = 5",
            table_name
        ))
        .fetch_one(&pool)
        .await?;
        let decoded_none: Option<TestableGeoType> = row_none.try_get("geom_null")?;
        assert_eq!(decoded_none, none_val);

        pool.execute(format!("DROP TABLE IF EXISTS {}", table_name).as_str())
            .await?;
        Ok(())
    }
}
