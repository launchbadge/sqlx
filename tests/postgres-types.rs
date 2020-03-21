extern crate time_ as time;

use std::sync::atomic::{AtomicU32, Ordering};

use sqlx::decode::Decode;
use sqlx::encode::Encode;
use sqlx::postgres::types::raw::{PgNumeric, PgNumericSign, PgRecordDecoder, PgRecordEncoder};
use sqlx::postgres::{PgQueryAs, PgTypeInfo, PgValue};
use sqlx::{Cursor, Executor, Postgres, Row, Type};
use sqlx_test::{new, test_prepared_type, test_type};

test_type!(null(
    Postgres,
    Option<i16>,
    "NULL" == None::<i16>
));

test_type!(bool(
    Postgres,
    bool,
    "false::boolean" == false,
    "true::boolean" == true
));

test_type!(i16(Postgres, i16, "821::smallint" == 821_i16));
test_type!(i32(Postgres, i32, "94101::int" == 94101_i32));
test_type!(i64(Postgres, i64, "9358295312::bigint" == 9358295312_i64));

test_type!(f32(Postgres, f32, "9419.122::real" == 9419.122_f32));
test_type!(f64(
    Postgres,
    f64,
    "939399419.1225182::double precision" == 939399419.1225182_f64
));

test_type!(string(
    Postgres,
    String,
    "'this is foo'" == "this is foo",
    "''" == ""
));

test_type!(bytea(
    Postgres,
    Vec<u8>,
    "E'\\\\xDEADBEEF'::bytea"
        == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "E'\\\\x'::bytea"
        == Vec::<u8>::new(),
    "E'\\\\x0000000052'::bytea"
        == vec![0_u8, 0, 0, 0, 0x52]
));

// PgNumeric only works on the wire protocol
test_prepared_type!(numeric(
    Postgres,
    PgNumeric,
    "0::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 0,
            scale: 0,
            digits: vec![]
        },
    "(-0)::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 0,
            scale: 0,
            digits: vec![]
        },
    "1::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 0,
            scale: 0,
            digits: vec![1]
        },
    "1234::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 0,
            scale: 0,
            digits: vec![1234]
        },
    "10000::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 1,
            scale: 0,
            digits: vec![1]
        },
    "0.1::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: -1,
            scale: 1,
            digits: vec![1000]
        },
    "0.01234::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: -1,
            scale: 5,
            digits: vec![123, 4000]
        },
    "12.34::numeric"
        == PgNumeric::Number {
            sign: PgNumericSign::Positive,
            weight: 0,
            scale: 2,
            digits: vec![12, 3400]
        },
    "'NaN'::numeric" == PgNumeric::NotANumber,
));

#[cfg(feature = "bigdecimal")]
test_type!(decimal(
    Postgres,
    sqlx::types::BigDecimal,
    "1::numeric" == "1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "10000::numeric" == "10000".parse::<sqlx::types::BigDecimal>().unwrap(),
    "0.1::numeric" == "0.1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "0.01234::numeric" == "0.01234".parse::<sqlx::types::BigDecimal>().unwrap(),
    "12.34::numeric" == "12.34".parse::<sqlx::types::BigDecimal>().unwrap(),
    "12345.6789::numeric" == "12345.6789".parse::<sqlx::types::BigDecimal>().unwrap(),
));

#[cfg(feature = "uuid")]
test_type!(uuid(
    Postgres,
    sqlx::types::Uuid,
    "'b731678f-636f-4135-bc6f-19440c13bd19'::uuid"
        == sqlx::types::Uuid::parse_str("b731678f-636f-4135-bc6f-19440c13bd19").unwrap(),
    "'00000000-0000-0000-0000-000000000000'::uuid"
        == sqlx::types::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
));

#[cfg(feature = "ipnetwork")]
test_type!(ipnetwork(
    Postgres,
    sqlx::types::ipnetwork::IpNetwork,
    "'127.0.0.1'::inet"
        == "127.0.0.1"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
    "'8.8.8.8/24'::inet"
        == "8.8.8.8/24"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
    "'::ffff:1.2.3.0'::inet"
        == "::ffff:1.2.3.0"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
    "'2001:4f8:3:ba::/64'::inet"
        == "2001:4f8:3:ba::/64"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
    "'192.168'::cidr"
        == "192.168.0.0/24"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
    "'::ffff:1.2.3.0/120'::cidr"
        == "::ffff:1.2.3.0/120"
            .parse::<sqlx::types::ipnetwork::IpNetwork>()
            .unwrap(),
));

#[cfg(feature = "chrono")]
mod chrono {
    use sqlx::types::chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    use super::*;

    test_type!(chrono_date(
        Postgres,
        NaiveDate,
        "DATE '2001-01-05'" == NaiveDate::from_ymd(2001, 1, 5),
        "DATE '2050-11-23'" == NaiveDate::from_ymd(2050, 11, 23)
    ));

    test_type!(chrono_time(
        Postgres,
        NaiveTime,
        "TIME '05:10:20.115100'" == NaiveTime::from_hms_micro(5, 10, 20, 115100)
    ));

    test_type!(chrono_date_time(
        Postgres,
        NaiveDateTime,
        "'2019-01-02 05:10:20'::timestamp" == NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20)
    ));

    test_type!(chrono_date_time_tz(
        Postgres,
        DateTime::<Utc>,
        "TIMESTAMPTZ '2019-01-02 05:10:20.115100'"
            == DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd(2019, 1, 2).and_hms_micro(5, 10, 20, 115100),
                Utc,
            )
    ));
}

#[cfg(feature = "time")]
mod time_tests {
    use super::*;
    use sqlx::types::time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
    use time::{date, time};

    test_type!(time_date(
        Postgres,
        Date,
        "DATE '2001-01-05'" == date!(2001 - 1 - 5),
        "DATE '2050-11-23'" == date!(2050 - 11 - 23)
    ));

    test_type!(time_time(
        Postgres,
        Time,
        "TIME '05:10:20.115100'" == time!(5:10:20.115100)
    ));

    test_type!(time_date_time(
        Postgres,
        PrimitiveDateTime,
        "TIMESTAMP '2019-01-02 05:10:20'" == date!(2019 - 1 - 2).with_time(time!(5:10:20)),
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2).with_time(time!(5:10:20.115100))
    ));

    test_type!(time_timestamp(
        Postgres,
        OffsetDateTime,
        "TIMESTAMPTZ '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2)
                .with_time(time!(5:10:20.115100))
                .assume_utc()
    ));
}

// This is trying to break my complete lack of understanding of null bitmaps for array/record
// decoding. The docs in pg are either wrong or I'm reading the wrong docs.
test_type!(lots_of_nulls_vec(Postgres, Vec<Option<bool>>,
    "ARRAY[NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, true]::bool[]" == {
      vec![None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(true)]
    },
));

test_type!(bool_vec(Postgres, Vec<bool>,
    "ARRAY[true, true, false, true]::bool[]" == vec![true, true, false, true],
));

test_type!(bool_opt_vec(Postgres, Vec<Option<bool>>,
    "ARRAY[NULL, true, NULL, false]::bool[]" == vec![None, Some(true), None, Some(false)],
));

test_type!(f32_vec(Postgres, Vec<f32>,
    "ARRAY[0.0, 1.0, 3.14, 1.234, -0.002, 100000.0]::real[]" == vec![0.0_f32, 1.0, 3.14, 1.234, -0.002, 100000.0],
));

test_type!(f64_vec(Postgres, Vec<f64>,
    "ARRAY[0.0, 1.0, 3.14, 1.234, -0.002, 100000.0]::double precision[]" == vec![0.0_f64, 1.0, 3.14, 1.234, -0.002, 100000.0],
));

test_type!(i16_vec(Postgres, Vec<i16>,
    "ARRAY[1, 152, -12412]::smallint[]" == vec![1_i16, 152, -12412],
    "ARRAY[]::smallint[]" == Vec::<i16>::new(),
    "ARRAY[0]::smallint[]" == vec![0_i16]
));

test_type!(string_vec(Postgres, Vec<String>,
    "ARRAY['', '\"']::text[]"
        == vec!["".to_string(), "\"".to_string()],

    "ARRAY['Hello, World', '', 'Goodbye']::text[]"
        == vec!["Hello, World".to_string(), "".to_string(), "Goodbye".to_string()],
));

//
// These require some annoyingly different tests as anonymous records cannot be read from the
// database. If someone enterprising comes along and wants to try and just the macro to handle
// this, that would be super awesome.
//

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_prepared_anonymous_record() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // Tuple of no elements is not possible
    // Tuple of 1 element requires a concrete type
    // Tuple with a NULL requires a concrete type

    // Tuple of 2 elements
    let rec: ((bool, i32),) = sqlx::query_as("SELECT (true, 23512)")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!((rec.0).0, true);
    assert_eq!((rec.0).1, 23512);

    // Tuple with an empty string
    let rec: ((bool, String),) = sqlx::query_as("SELECT (true,'')")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!((rec.0).1, "");

    // Tuple with a string with an interior comma
    let rec: ((bool, String),) = sqlx::query_as("SELECT (true,'Hello, World!')")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!((rec.0).1, "Hello, World!");

    // Tuple with a string with an emoji
    let rec: ((bool, String),) = sqlx::query_as("SELECT (true,'Hello, 🐕!')")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!((rec.0).1, "Hello, 🐕!");

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_unprepared_anonymous_record() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // Tuple of no elements is not possible
    // Tuple of 1 element requires a concrete type
    // Tuple with a NULL requires a concrete type

    // Tuple of 2 elements
    let mut cursor = conn.fetch("SELECT (true, 23512)");
    let row = cursor.next().await?.unwrap();
    let rec: (bool, i32) = row.get(0);

    assert_eq!(rec.0, true);
    assert_eq!(rec.1, 23512);

    // Tuple with an empty string
    let mut cursor = conn.fetch("SELECT (true, '')");
    let row = cursor.next().await?.unwrap();
    let rec: (bool, String) = row.get(0);

    assert_eq!(rec.1, "");

    // Tuple with a string with an interior comma
    let mut cursor = conn.fetch("SELECT (true, 'Hello, World!')");
    let row = cursor.next().await?.unwrap();
    let rec: (bool, String) = row.get(0);

    assert_eq!(rec.1, "Hello, World!");

    // Tuple with a string with an emoji
    let mut cursor = conn.fetch("SELECT (true, 'Hello, 🐕!')");
    let row = cursor.next().await?.unwrap();
    let rec: (bool, String) = row.get(0);

    assert_eq!(rec.1, "Hello, 🐕!");

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_unprepared_anonymous_record_arrays() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // record of arrays
    let mut cursor = conn.fetch("SELECT (ARRAY['', '\"']::text[], false)");
    let row = cursor.next().await?.unwrap();
    let rec: (Vec<String>, bool) = row.get(0);

    assert_eq!(rec, (vec!["".to_string(), "\"".to_string()], false));

    // array of records
    let mut cursor = conn.fetch("SELECT ARRAY[('','\"'), (NULL,'')]::record[]");
    let row = cursor.next().await?.unwrap();
    let rec: Vec<(Option<String>, String)> = row.get(0);

    assert_eq!(
        rec,
        vec![
            (Some(String::from("")), String::from("\"")),
            (None, String::from(""))
        ]
    );

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_prepared_anonymous_record_arrays() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // record of arrays
    let rec: ((Vec<String>, bool),) = sqlx::query_as("SELECT (ARRAY['', '\"']::text[], false)")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec.0, (vec!["".to_string(), "\"".to_string()], false));

    // array of records
    let rec: (Vec<(Option<String>, String)>,) =
        sqlx::query_as("SELECT ARRAY[('','\"'), (NULL,'')]::record[]")
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(
        rec.0,
        vec![
            (Some(String::from("")), String::from("\"")),
            (None, String::from(""))
        ]
    );

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_prepared_structs() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    //
    // Setup custom types if needed
    //

    static OID_RECORD_EMPTY: AtomicU32 = AtomicU32::new(0);
    static OID_RECORD_1: AtomicU32 = AtomicU32::new(0);

    conn.execute(
        r#"
DO $$ BEGIN
    CREATE TYPE _sqlx_record_empty AS ();
    CREATE TYPE _sqlx_record_1 AS (_1 int8);
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
    "#,
    )
    .await?;

    let type_ids: Vec<(i32,)> = sqlx::query_as(
        "SELECT oid::int4 FROM pg_type WHERE typname IN ('_sqlx_record_empty', '_sqlx_record_1')",
    )
    .fetch_all(&mut conn)
    .await?;

    OID_RECORD_EMPTY.store(type_ids[0].0 as u32, Ordering::SeqCst);
    OID_RECORD_1.store(type_ids[1].0 as u32, Ordering::SeqCst);

    //
    // Record of no elements
    //

    struct RecordEmpty {}

    impl Type<Postgres> for RecordEmpty {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(OID_RECORD_EMPTY.load(Ordering::SeqCst))
        }
    }

    impl Encode<Postgres> for RecordEmpty {
        fn encode(&self, buf: &mut Vec<u8>) {
            PgRecordEncoder::new(buf).finish();
        }
    }

    impl<'de> Decode<'de, Postgres> for RecordEmpty {
        fn decode(_value: Option<PgValue<'de>>) -> sqlx::Result<Postgres, Self> {
            Ok(RecordEmpty {})
        }
    }

    let _: (RecordEmpty, RecordEmpty) = sqlx::query_as("SELECT '()'::_sqlx_record_empty, $1")
        .bind(RecordEmpty {})
        .fetch_one(&mut conn)
        .await?;

    //
    // Record of one element
    //

    #[derive(Debug, PartialEq)]
    struct Record1 {
        _1: i64,
    }

    impl Type<Postgres> for Record1 {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_oid(OID_RECORD_1.load(Ordering::SeqCst))
        }
    }

    impl Encode<Postgres> for Record1 {
        fn encode(&self, buf: &mut Vec<u8>) {
            PgRecordEncoder::new(buf).encode(self._1).finish();
        }
    }

    impl<'de> Decode<'de, Postgres> for Record1 {
        fn decode(value: Option<PgValue<'de>>) -> sqlx::Result<Postgres, Self> {
            let mut decoder = PgRecordDecoder::new(value)?;

            let _1 = decoder.decode()?;

            Ok(Record1 { _1 })
        }
    }

    let rec: (Record1, Record1) = sqlx::query_as("SELECT '(324235)'::_sqlx_record_1, $1")
        .bind(Record1 { _1: 324235 })
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec.0, rec.1);

    Ok(())
}

//
// JSON
//

#[cfg(feature = "json")]
mod json {
    use super::*;
    use serde_json::value::RawValue;
    use serde_json::{json, Value as JsonValue};
    use sqlx::postgres::types::PgJson;
    use sqlx::postgres::PgRow;
    use sqlx::types::Json;
    use sqlx::Row;

    // When testing JSON, coerce to JSONB for `=` comparison as `JSON = JSON` is not
    // supported in PostgreSQL

    test_type!(json(
        Postgres,
        PgJson<JsonValue>,
        "SELECT {0}::jsonb is not distinct from $1::jsonb, $2::text as _1, {0} as _2, $3 as _3",
        "'\"Hello, World\"'::json" == PgJson(json!("Hello, World")),
        "'\"😎\"'::json" == PgJson(json!("😎")),
        "'\"🙋‍♀️\"'::json" == PgJson(json!("🙋‍♀️")),
        "'[\"Hello\", \"World!\"]'::json" == PgJson(json!(["Hello", "World!"]))
    ));

    test_type!(jsonb(
        Postgres,
        JsonValue,
        "'\"Hello, World\"'::jsonb" == json!("Hello, World"),
        "'\"😎\"'::jsonb" == json!("😎"),
        "'\"🙋‍♀️\"'::jsonb" == json!("🙋‍♀️"),
        "'[\"Hello\", \"World!\"]'::jsonb" == json!(["Hello", "World!"])
    ));

    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct Friend {
        name: String,
        age: u32,
    }

    // The default JSON type that SQLx chooses is JSONB
    //  sqlx::types::Json -> JSONB
    //  sqlx::postgres::types::PgJson -> JSON
    //  sqlx::postgres::types::PgJsonB -> JSONB

    test_type!(jsonb_struct(Postgres, Json<Friend>,
        "'{\"name\":\"Joe\",\"age\":33}'::jsonb" == Json(Friend { name: "Joe".to_string(), age: 33 })
    ));

    test_type!(json_struct(
        Postgres,
        PgJson<Friend>,
        "SELECT {0}::jsonb is not distinct from $1::jsonb, $2::text as _1, {0} as _2, $3 as _3",
        "'{\"name\":\"Joe\",\"age\":33}'::json" == PgJson(Friend { name: "Joe".to_string(), age: 33 })
    ));

    #[cfg_attr(feature = "runtime-async-std", async_std::test)]
    #[cfg_attr(feature = "runtime-tokio", tokio::test)]
    async fn test_prepared_jsonb_raw_value() -> anyhow::Result<()> {
        let mut conn = new::<Postgres>().await?;

        let mut cursor = sqlx::query("SELECT '{\"hello\": \"world\"}'::jsonb").fetch(&mut conn);
        let row: PgRow = cursor.next().await?.unwrap();
        let value: &RawValue = row.get::<&RawValue, usize>(0_usize);

        assert_eq!(value.get(), "{\"hello\": \"world\"}");

        Ok(())
    }
}
