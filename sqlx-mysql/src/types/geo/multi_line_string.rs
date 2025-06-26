use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

use crate::io::MySqlBufMutExt;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{MySql, MySqlTypeInfo, MySqlValueRef};

use std::convert::TryFrom;

use geo_traits::to_geo::ToGeoGeometry;
use geo_types::MultiLineString;
use wkb::reader;
use wkb::writer;

impl Type<MySql> for MultiLineString {
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

impl Encode<'_, MySql> for MultiLineString {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        let mut wkb_buffer = Vec::new();
        wkb_buffer.extend_from_slice(&0u32.to_le_bytes()); // SRID = 0, Little Endian
        writer::write_multi_line_string(&mut wkb_buffer, self, &writer::WriteOptions::default())?;
        buf.put_bytes_lenenc(&wkb_buffer);
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, MySql> for MultiLineString {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        if bytes.len() < 4 {
            return Err(format!(
                "Invalid GEOMETRY data for MultiLineString: received {} bytes, expected at least 4 for SRID prefix.",
                bytes.len()
            )
            .into());
        }
        let wkb_data = &bytes[4..]; // Skip 4-byte SRID

        let wkb_reader_geom = reader::Wkb::try_new(wkb_data).map_err(|e| {
            BoxDynError::from(format!("WKB parsing error for MultiLineString: {}", e))
        })?;

        let geo_geom: geo_types::Geometry<f64> = wkb_reader_geom.to_geometry();

        MultiLineString::try_from(geo_geom).map_err(|e| {
            BoxDynError::from(format!(
                "Failed to convert geo_types::Geometry to MultiLineString: {:?}",
                e
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use geo_types::coord;
    use geo_types::LineString;
    use geo_types::MultiLineString as TestableGeoType;
    use sqlx::mysql::{MySqlPool, MySqlRow};
    use sqlx::{Executor, Row};

    #[sqlx::test]
    async fn test_encode_decode_multilinestring(pool: MySqlPool) -> anyhow::Result<()> {
        let table_name = format!("test_geo_multilinestring_table");
        pool.execute(format!("DROP TABLE IF EXISTS {}", table_name).as_str())
            .await?;
        pool.execute(
            format!(
                "CREATE TABLE {} (id INT, geom GEOMETRY, geom_null GEOMETRY NULL)",
                table_name
            )
            .as_str(),
        )
        .await?;

        let ls1_in = LineString::new(vec![coord! {x:0.,y:0.}, coord! {x:1.,y:1.}]);
        let ls2_in = LineString::new(vec![coord! {x:2.,y:2.}, coord! {x:3.,y:3.}]);
        let mls1 = TestableGeoType::new(vec![ls1_in, ls2_in]);
        let ls3_in = LineString::new(vec![coord! {x:10.,y:10.}, coord! {x:20.,y:20.}]);
        let mls2 = TestableGeoType::new(vec![ls3_in]);

        // Test non-nullable
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom) VALUES (1, ?)",
            table_name
        ))
        .bind(mls1.clone())
        .execute(&pool)
        .await?;

        let row: MySqlRow = sqlx::query(&format!("SELECT geom FROM {} WHERE id = 1", table_name))
            .fetch_one(&pool)
            .await?;
        let decoded_val: TestableGeoType = row.try_get("geom")?;
        assert_eq!(decoded_val, mls1);

        // Test nullable Some(value)
        let some_val: Option<TestableGeoType> = Some(mls2.clone());
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_null) VALUES (2, ?)",
            table_name
        ))
        .bind(some_val.clone())
        .execute(&pool)
        .await?;

        let row_some: MySqlRow = sqlx::query(&format!(
            "SELECT geom_null FROM {} WHERE id = 2",
            table_name
        ))
        .fetch_one(&pool)
        .await?;
        let decoded_some: Option<TestableGeoType> = row_some.try_get("geom_null")?;
        assert_eq!(decoded_some, some_val);

        // Test nullable None
        let none_val: Option<TestableGeoType> = None;
        sqlx::query(&format!(
            "INSERT INTO {} (id, geom_null) VALUES (3, ?)",
            table_name
        ))
        .bind(none_val.clone())
        .execute(&pool)
        .await?;

        let row_none: MySqlRow = sqlx::query(&format!(
            "SELECT geom_null FROM {} WHERE id = 3",
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
