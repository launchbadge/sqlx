extern crate time_ as time;

use std::net::SocketAddr;
#[cfg(feature = "rust_decimal")]
use std::str::FromStr;

use sqlx::mysql::MySql;
use sqlx::{Executor, Row};

use sqlx::types::Text;

use sqlx_test::{new, test_type};

test_type!(bool(MySql, "false" == false, "true" == true));

test_type!(u8(MySql, "CAST(253 AS UNSIGNED)" == 253_u8));
test_type!(i8(MySql, "5" == 5_i8, "0" == 0_i8));

test_type!(u16(MySql, "CAST(21415 AS UNSIGNED)" == 21415_u16));
test_type!(i16(MySql, "21415" == 21415_i16));

test_type!(u32(MySql, "CAST(2141512 AS UNSIGNED)" == 2141512_u32));
test_type!(i32(MySql, "2141512" == 2141512_i32));

test_type!(u64(MySql, "CAST(2141512 AS UNSIGNED)" == 2141512_u64));
test_type!(i64(MySql, "2141512" == 2141512_i64));

test_type!(f64(MySql, "3.14159265e0" == 3.14159265_f64));

// NOTE: This behavior can be very surprising. MySQL implicitly widens FLOAT bind parameters
//       to DOUBLE. This results in the weirdness you see below. MySQL generally recommends to stay
//       away from FLOATs.
test_type!(f32(MySql, "3.1410000324249268e0" == 3.141f32 as f64 as f32));

test_type!(string<String>(MySql,
    "'helloworld'" == "helloworld",
    "''" == ""
));

test_type!(bytes<Vec<u8>>(MySql,
    "X'DEADBEEF'"
        == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "X''"
        == Vec::<u8>::new(),
    "X'0000000052'"
        == vec![0_u8, 0, 0, 0, 0x52]
));

#[cfg(feature = "uuid")]
test_type!(uuid<sqlx::types::Uuid>(MySql,
    "x'b731678f636f4135bc6f19440c13bd19'"
        == sqlx::types::Uuid::parse_str("b731678f-636f-4135-bc6f-19440c13bd19").unwrap(),
    "x'00000000000000000000000000000000'"
        == sqlx::types::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
));

#[cfg(feature = "uuid")]
test_type!(uuid_hyphenated<sqlx::types::uuid::fmt::Hyphenated>(MySql,
    "'b731678f-636f-4135-bc6f-19440c13bd19'"
        == sqlx::types::Uuid::parse_str("b731678f-636f-4135-bc6f-19440c13bd19").unwrap().hyphenated(),
    "'00000000-0000-0000-0000-000000000000'"
        == sqlx::types::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap().hyphenated()
));

#[cfg(feature = "uuid")]
test_type!(uuid_simple<sqlx::types::uuid::fmt::Simple>(MySql,
    "'b731678f636f4135bc6f19440c13bd19'"
        == sqlx::types::Uuid::parse_str("b731678f636f4135bc6f19440c13bd19").unwrap().simple(),
    "'00000000000000000000000000000000'"
        == sqlx::types::Uuid::parse_str("00000000000000000000000000000000").unwrap().simple()
));

#[cfg(feature = "chrono")]
mod chrono {
    use sqlx::types::chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    use super::*;

    test_type!(chrono_date<NaiveDate>(MySql,
        "DATE '2001-01-05'" == NaiveDate::from_ymd(2001, 1, 5),
        "DATE '2050-11-23'" == NaiveDate::from_ymd(2050, 11, 23)
    ));

    test_type!(chrono_time_zero<NaiveTime>(MySql,
        "TIME '00:00:00.000000'" == NaiveTime::from_hms_micro(0, 0, 0, 0)
    ));

    test_type!(chrono_time<NaiveTime>(MySql,
        "TIME '05:10:20.115100'" == NaiveTime::from_hms_micro(5, 10, 20, 115100)
    ));

    test_type!(chrono_date_time<NaiveDateTime>(MySql,
        "TIMESTAMP '2019-01-02 05:10:20'" == NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20)
    ));

    test_type!(chrono_timestamp<DateTime::<Utc>>(MySql,
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd(2019, 1, 2).and_hms_micro(5, 10, 20, 115100),
                Utc,
            )
    ));

    #[sqlx_macros::test]
    async fn test_type_chrono_zero_date() -> anyhow::Result<()> {
        let mut conn = sqlx_test::new::<MySql>().await?;

        // ensure that zero dates are turned on
        // newer MySQL has these disabled by default

        conn.execute("SET @@sql_mode := REPLACE(@@sql_mode, 'NO_ZERO_IN_DATE', '');")
            .await?;

        conn.execute("SET @@sql_mode := REPLACE(@@sql_mode, 'NO_ZERO_DATE', '');")
            .await?;

        // date

        let row = sqlx::query("SELECT DATE '0000-00-00'")
            .fetch_one(&mut conn)
            .await?;

        let val: Option<NaiveDate> = row.get(0);

        assert_eq!(val, None);
        assert!(row.try_get::<NaiveDate, _>(0).is_err());

        // datetime

        let row = sqlx::query("SELECT TIMESTAMP '0000-00-00 00:00:00'")
            .fetch_one(&mut conn)
            .await?;

        let val: Option<NaiveDateTime> = row.get(0);

        assert_eq!(val, None);
        assert!(row.try_get::<NaiveDateTime, _>(0).is_err());

        Ok(())
    }
}

#[cfg(feature = "time")]
mod time_tests {
    use time::macros::{date, time};

    use sqlx::types::time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

    use super::*;

    test_type!(time_date<Date>(
        MySql,
        "DATE '2001-01-05'" == date!(2001 - 1 - 5),
        "DATE '2050-11-23'" == date!(2050 - 11 - 23)
    ));

    test_type!(time_time_zero<Time>(
        MySql,
        "TIME '00:00:00.000000'" == time!(00:00:00.000000)
    ));

    test_type!(time_time<Time>(
        MySql,
        "TIME '05:10:20.115100'" == time!(5:10:20.115100)
    ));

    test_type!(time_date_time<PrimitiveDateTime>(
        MySql,
        "TIMESTAMP '2019-01-02 05:10:20'" == date!(2019 - 1 - 2).with_time(time!(5:10:20)),
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2).with_time(time!(5:10:20.115100))
    ));

    test_type!(time_timestamp<OffsetDateTime>(
        MySql,
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2)
                .with_time(time!(5:10:20.115100))
                .assume_utc()
    ));

    #[sqlx_macros::test]
    async fn test_type_time_zero_date() -> anyhow::Result<()> {
        let mut conn = sqlx_test::new::<MySql>().await?;

        // ensure that zero dates are turned on
        // newer MySQL has these disabled by default

        conn.execute("SET @@sql_mode := REPLACE(@@sql_mode, 'NO_ZERO_IN_DATE', '');")
            .await?;

        conn.execute("SET @@sql_mode := REPLACE(@@sql_mode, 'NO_ZERO_DATE', '');")
            .await?;

        // date

        let row = sqlx::query("SELECT DATE '0000-00-00'")
            .fetch_one(&mut conn)
            .await?;

        let val: Option<Date> = row.get(0);

        assert_eq!(val, None);
        assert!(row.try_get::<Date, _>(0).is_err());

        // datetime

        let row = sqlx::query("SELECT TIMESTAMP '0000-00-00 00:00:00'")
            .fetch_one(&mut conn)
            .await?;

        let val: Option<PrimitiveDateTime> = row.get(0);

        assert_eq!(val, None);
        assert!(row.try_get::<PrimitiveDateTime, _>(0).is_err());

        Ok(())
    }
}

#[cfg(feature = "bigdecimal")]
test_type!(bigdecimal<sqlx::types::BigDecimal>(
    MySql,
    "CAST(0 as DECIMAL(0, 0))" == "0".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(1 AS DECIMAL(1, 0))" == "1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(10000 AS DECIMAL(5, 0))" == "10000".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(0.1 AS DECIMAL(2, 1))" == "0.1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(0.01234 AS DECIMAL(6, 5))" == "0.01234".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(12.34 AS DECIMAL(4, 2))" == "12.34".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(12345.6789 AS DECIMAL(9, 4))" == "12345.6789".parse::<sqlx::types::BigDecimal>().unwrap(),
));

#[cfg(feature = "rust_decimal")]
test_type!(decimal<sqlx::types::Decimal>(MySql,
    "CAST(0 as DECIMAL(0, 0))" == sqlx::types::Decimal::from_str("0").unwrap(),
    "CAST(1 AS DECIMAL(1, 0))" == sqlx::types::Decimal::from_str("1").unwrap(),
    "CAST(10000 AS DECIMAL(5, 0))" == sqlx::types::Decimal::from_str("10000").unwrap(),
    "CAST(0.1 AS DECIMAL(2, 1))" == sqlx::types::Decimal::from_str("0.1").unwrap(),
    "CAST(0.01234 AS DECIMAL(6, 5))" == sqlx::types::Decimal::from_str("0.01234").unwrap(),
    "CAST(12.34 AS DECIMAL(4, 2))" == sqlx::types::Decimal::from_str("12.34").unwrap(),
    "CAST(12345.6789 AS DECIMAL(9, 4))" == sqlx::types::Decimal::from_str("12345.6789").unwrap(),
));

#[cfg(feature = "json")]
mod json_tests {
    use serde_json::{json, Value as JsonValue};

    use sqlx::types::Json;
    use sqlx_test::test_type;

    use super::*;

    test_type!(json<JsonValue>(
        MySql,
        // MySQL 8.0.27 changed `<=>` to return an unsigned integer
        "SELECT CAST(CAST({0} AS BINARY) <=> CAST(? AS BINARY) AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "'\"Hello, World\"'" == json!("Hello, World"),
        "'\"😎\"'" == json!("😎"),
        "'\"🙋‍♀️\"'" == json!("🙋‍♀️"),
        "'[\"Hello\",\"World!\"]'" == json!(["Hello", "World!"])
    ));

    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct Friend {
        name: String,
        age: u32,
    }

    test_type!(json_struct<Json<Friend>>(
        MySql,
        // MySQL 8.0.27 changed `<=>` to return an unsigned integer
        "SELECT CAST(CAST({0} AS BINARY) <=> CAST(? AS BINARY) AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "\'{\"name\":\"Joe\",\"age\":33}\'" == Json(Friend { name: "Joe".to_string(), age: 33 })
    ));

    // NOTE: This is testing recursive (and transparent) usage of the `Json` wrapper. You don't
    //       need to wrap the Vec in Json<_> to make the example work.

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Customer {
        json_column: Json<Vec<i64>>,
    }

    test_type!(json_struct_json_column<Json<Customer>>(
        MySql,
        "\'{\"json_column\":[1,2]}\'" == Json(Customer { json_column: Json(vec![1, 2]) })
    ));
}

#[cfg(feature = "geometry")]
mod geometry_tests {
    use geo_types::{
        line_string, point, polygon, Geometry, GeometryCollection, LineString, MultiPoint, Point,
        Polygon,
    };
    use sqlx_test::test_type;

    use super::*;

    test_type!(geometry_point<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('POINT(1 1)')" == Geometry::Point(point!( x: 1.0, y: 1.0 )),
    ));

    test_type!(geometry_subtype_point<Point<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('POINT(3 4)')" == point!( x: 3.0, y: 4.0 ),
    ));

    test_type!(geometry_linestring<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('LINESTRING(0 0, 1 1, 2 2)')" == Geometry::LineString(line_string![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 2.0, y: 2.0),
        ]),
    ));

    test_type!(geometry_subtype_linestring<LineString<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('LINESTRING(6 5, 4 3, 2 1)')" == line_string![
            (x: 6.0, y: 5.0),
            (x: 4.0, y: 3.0),
            (x: 2.0, y: 1.0),
        ],
    ));

    test_type!(geometry_polygon<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('POLYGON((0 0, 1 1, 1 0, 0 0))')" == Geometry::Polygon(polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 1.0, y: 0.0),
            (x: 0.0, y: 0.0),
        ]),
    ));

    test_type!(geometry_subtype_polygon<Polygon<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('POLYGON((0 0, 2 2, 2 0, 0 0))')" == polygon![
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 2.0),
            (x: 2.0, y: 0.0),
            (x: 0.0, y: 0.0),
        ],
    ));

    test_type!(geometry_multipoint<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('MULTIPOINT(0 0, 1 1, 2 2)')" == Geometry::MultiPoint(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ].into()),
    ));

    test_type!(geometry_subtype_multipoint<MultiPoint<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('MULTIPOINT(0 0, 1 1, 2 2)')" == MultiPoint(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ]),
    ));

    test_type!(geometry_collection<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(0 0, 1 1, 2 2),POLYGON((0 0, 1 1, 1 0, 0 0)))')" == Geometry::GeometryCollection(GeometryCollection(vec![
            Geometry::Point(point!(x: 1.0, y: 1.0)),
            Geometry::LineString(line_string![
                (x: 0.0, y: 0.0),
                (x: 1.0, y: 1.0),
                (x: 2.0, y: 2.0),
            ]),
            Geometry::Polygon(polygon![
                (x: 0.0, y: 0.0),
                (x: 1.0, y: 1.0),
                (x: 1.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ]),
        ])),
    ));

    test_type!(geometry_subtype_collection<GeometryCollection<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('GEOMETRYCOLLECTION(POINT(8 7),LINESTRING(6 5, 4 3, 2 1),POLYGON((0 0, 1 1, 1 0, 0 0)))')" == GeometryCollection(vec![
            Geometry::Point(point!(x: 8.0, y: 7.0)),
            Geometry::LineString(line_string![
                (x: 6.0, y: 5.0),
                (x: 4.0, y: 3.0),
                (x: 2.0, y: 1.0),
            ]),
            Geometry::Polygon(polygon![
                (x: 0.0, y: 0.0),
                (x: 1.0, y: 1.0),
                (x: 1.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ]),
        ]),
    ));

    test_type!(geometry_collection_empty<Geometry<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('GEOMETRYCOLLECTION EMPTY')" == Geometry::<f64>::GeometryCollection(GeometryCollection(vec![])),
    ));

    test_type!(geometry_subtype_collection_empty<GeometryCollection<f64>>(
        MySql,
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), CAST({0} AS BINARY) as _2, ? as _3",
        "ST_GeomFromText('GEOMETRYCOLLECTION EMPTY')" == GeometryCollection::<f64>(vec![]),
    ));

    macro_rules! geo_table {
        (CREATE, $ty:literal) => {
            format!(
                r#"
CREATE TEMPORARY TABLE with_geometry (
    id INT PRIMARY KEY AUTO_INCREMENT,
    geom {} NOT NULL
);"#,
                $ty
            )
        };
        (TRUNCATE) => {
            "TRUNCATE TABLE with_geometry"
        };
    }

    /// Test with a table that has a column which type is a subtype of `GEOMETRY`
    ///
    /// It tests that we can insert and select values from the table, including using
    /// geometry literals in selection.
    ///
    /// Because of the limitations of MySQL, we have to use the `Blob` type to represent
    /// the [`Geometry`] type, so use case testing in actual tables make more sense with
    /// the actual use of users.
    macro_rules! test_geo_table {
        ($name:ident, $col:literal, $($text:literal == <$ty:ty>$value:expr),+ $(,)?) => {
            paste::item! {
                #[sqlx_macros::test]
                async fn [< test_geometry_table_ $name >] () -> anyhow::Result<()> {
                    use sqlx::Connection;

                    let mut conn = sqlx_test::new::<MySql>().await?;
                    let tdl = geo_table!(CREATE, $col);

                    conn.execute(tdl.as_str()).await?;

                    $(
                        let expected = $value;

                        println!("Insert with select {:?}", expected);
                        sqlx::query("INSERT INTO with_geometry (geom) VALUES (?)")
                            .bind(&expected)
                            .execute(&mut conn)
                            .await?;

                        let row = sqlx::query("SELECT geom FROM with_geometry WHERE geom = ?")
                            .bind(&expected)
                            .fetch_one(&mut conn)
                            .await?;
                        let geom: $ty = row.try_get(0)?;

                        assert_eq!(geom, expected);

                        let query = format!("SELECT geom FROM with_geometry WHERE geom = {}", $text);
                        println!("{query}");

                        let row = sqlx::query(&query)
                            .fetch_one(&mut conn)
                            .await?;
                        let geom: $ty = row.try_get(0)?;

                        assert_eq!(geom, expected);
                        conn.execute(geo_table!(TRUNCATE)).await?;
                    )+

                    conn.close().await?;

                    Ok(())
                }
            }
        };
    }

    test_geo_table!(
        point,
        "POINT",
        "ST_GeomFromText('Point(0 0)')" == <Geometry<f64>>Geometry::Point(point!(x: 0.0, y: 0.0)),
        "ST_GeomFromText('Point(-2 -3)')" == <Geometry<f64>>Geometry::Point(point!(x: -2.0, y: -3.0)),
        "ST_GeomFromText('Point(5.76814 12345)')"
            == <Geometry<f64>>Geometry::Point(point!(x: 5.76814, y: 12345.0)),
        "ST_GeomFromText('Point(0 0)')"
            == <Point<f64>>point!(x: 0.0, y: 0.0),
        "ST_GeomFromText('Point(-5.7 -4.3)')" == <Point<f64>>point!(x: -5.7, y: -4.3),
    );

    test_geo_table!(
        linestring,
        "LINESTRING",
        "ST_GeomFromText('LineString(0 0, 1 1, 2 2)')"
            == <Geometry<f64>>Geometry::LineString(line_string![
                (x: 0.0, y: 0.0),
                (x: 1.0, y: 1.0),
                (x: 2.0, y: 2.0),
            ]),
        "ST_GeomFromText('LineString(6 5, 4 3, 2 1)')"
            == <LineString<f64>>line_string![
                (x: 6.0, y: 5.0),
                (x: 4.0, y: 3.0),
                (x: 2.0, y: 1.0),
            ],
    );

    test_geo_table!(
        polygon,
        "POLYGON",
        "ST_GeomFromText('Polygon((0 0, 1 1, 1 0, 0 0))')"
            == <Geometry<f64>>Geometry::Polygon(polygon![
                (x: 0.0, y: 0.0),
                (x: 1.0, y: 1.0),
                (x: 1.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ]),
        "ST_GeomFromText('Polygon((0 0, 2 2, 2 0, 0 0))')"
            == <Polygon<f64>>polygon![
                (x: 0.0, y: 0.0),
                (x: 2.0, y: 2.0),
                (x: 2.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ],
    );

    test_geo_table!(
        geometry_collection,
        "GEOMETRYCOLLECTION",
        "ST_GeomFromText('GeometryCollection(Point(1 2),LineString(3 4, 5 6, 7 8),Polygon((0 0, 1 1, 1 0, 0 0)))')"
            == <Geometry<f64>>Geometry::GeometryCollection(GeometryCollection(vec![
                Geometry::Point(point!(x: 1.0, y: 2.0)),
                Geometry::LineString(line_string![
                    (x: 3.0, y: 4.0),
                    (x: 5.0, y: 6.0),
                    (x: 7.0, y: 8.0),
                ]),
                Geometry::Polygon(polygon![
                    (x: 0.0, y: 0.0),
                    (x: 1.0, y: 1.0),
                    (x: 1.0, y: 0.0),
                    (x: 0.0, y: 0.0),
                ]),
            ])),
        "ST_GeomFromText('GeometryCollection(Point(8 7),LineString(6 5, 4 3, 2 1),Polygon((0 0, 1 1, 1 0, 0 0)))')"
            == <GeometryCollection<f64>>GeometryCollection(vec![
                Geometry::Point(point!(x: 8.0, y: 7.0)),
                Geometry::LineString(line_string![
                    (x: 6.0, y: 5.0),
                    (x: 4.0, y: 3.0),
                    (x: 2.0, y: 1.0),
                ]),
                Geometry::Polygon(polygon![
                    (x: 0.0, y: 0.0),
                    (x: 1.0, y: 1.0),
                    (x: 1.0, y: 0.0),
                    (x: 0.0, y: 0.0),
                ]),
            ]),
    );
}

#[sqlx_macros::test]
async fn test_bits() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    conn.execute(
        r#"
CREATE TEMPORARY TABLE with_bits (
    id INT PRIMARY KEY AUTO_INCREMENT,
    value_1 BIT(1) NOT NULL,
    value_n BIT(64) NOT NULL
);
    "#,
    )
    .await?;

    sqlx::query("INSERT INTO with_bits (value_1, value_n) VALUES (?, ?)")
        .bind(&1_u8)
        .bind(&510202_u32)
        .execute(&mut conn)
        .await?;

    // BINARY
    let (v1, vn): (u8, u64) = sqlx::query_as("SELECT value_1, value_n FROM with_bits")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(v1, 1);
    assert_eq!(vn, 510202);

    // TEXT
    let row = conn
        .fetch_one("SELECT value_1, value_n FROM with_bits")
        .await?;
    let v1: u8 = row.try_get(0)?;
    let vn: u64 = row.try_get(1)?;

    assert_eq!(v1, 1);
    assert_eq!(vn, 510202);

    Ok(())
}

#[sqlx_macros::test]
async fn test_text_adapter() -> anyhow::Result<()> {
    #[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
    struct Login {
        user_id: i32,
        socket_addr: Text<SocketAddr>,
        #[cfg(feature = "time")]
        login_at: time::OffsetDateTime,
    }

    let mut conn = new::<MySql>().await?;

    conn.execute(
        r#"
CREATE TEMPORARY TABLE user_login (
    user_id INT PRIMARY KEY AUTO_INCREMENT,
    socket_addr TEXT NOT NULL,
    login_at TIMESTAMP NOT NULL
);
    "#,
    )
    .await?;

    let user_id = 1234;
    let socket_addr: SocketAddr = "198.51.100.47:31790".parse().unwrap();

    sqlx::query("INSERT INTO user_login (user_id, socket_addr, login_at) VALUES (?, ?, NOW())")
        .bind(user_id)
        .bind(Text(socket_addr))
        .execute(&mut conn)
        .await?;

    let last_login: Login =
        sqlx::query_as("SELECT * FROM user_login ORDER BY login_at DESC LIMIT 1")
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(last_login.user_id, user_id);
    assert_eq!(*last_login.socket_addr, socket_addr);

    Ok(())
}
