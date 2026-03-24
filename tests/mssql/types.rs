extern crate time_ as time;

use sqlx::mssql::Mssql;
use sqlx_test::test_type;

test_type!(null<Option<i32>>(Mssql,
    "CAST(NULL as INT)" == None::<i32>
));

test_type!(u8(
    Mssql,
    "CAST(5 AS TINYINT)" == 5_u8,
    "CAST(0 AS TINYINT)" == 0_u8,
    "CAST(255 AS TINYINT)" == 255_u8,
));

test_type!(i8(
    Mssql,
    "CAST(5 AS TINYINT)" == 5_i8,
    "CAST(0 AS TINYINT)" == 0_i8
));

test_type!(i16(
    Mssql,
    "CAST(21415 AS SMALLINT)" == 21415_i16,
    "CAST(-32768 AS SMALLINT)" == i16::MIN,
    "CAST(32767 AS SMALLINT)" == i16::MAX,
));

test_type!(i32(
    Mssql,
    "CAST(2141512 AS INT)" == 2141512_i32,
    "CAST(-2147483648 AS INT)" == i32::MIN,
    "CAST(2147483647 AS INT)" == i32::MAX,
));

test_type!(i64(
    Mssql,
    "CAST(32324324432 AS BIGINT)" == 32324324432_i64,
    "CAST(-9223372036854775808 AS BIGINT)" == i64::MIN,
    "CAST(9223372036854775807 AS BIGINT)" == i64::MAX,
));

test_type!(f32(
    Mssql,
    "CAST(3.1410000324249268 AS REAL)" == 3.141f32 as f64 as f32
));

test_type!(f64(
    Mssql,
    "CAST(939399419.1225182 AS FLOAT)" == 939399419.1225182_f64
));

test_type!(f64_money<f64>(
    Mssql,
    "CAST(922337203685477.5807 AS MONEY)" == 922337203685477.5807_f64,
    "CAST(0 AS MONEY)" == 0.0_f64,
    "CAST(-1234.5678 AS MONEY)" == -1234.5678_f64,
));

test_type!(f64_smallmoney<f64>(
    Mssql,
    "CAST(214748.3647 AS SMALLMONEY)" == 214748.3647_f64,
    "CAST(0 AS SMALLMONEY)" == 0.0_f64,
    "CAST(-1234.5678 AS SMALLMONEY)" == -1234.5678_f64,
));

#[cfg(feature = "rust_decimal")]
test_type!(rust_decimal_smallmoney<sqlx::types::Decimal>(Mssql,
    "CAST(214748.3647 AS SMALLMONEY)" == sqlx::types::Decimal::new(2147483647, 4),
    "CAST(0 AS SMALLMONEY)" == sqlx::types::Decimal::ZERO,
));

test_type!(str_nvarchar<String>(Mssql,
    "CAST('this is foo' as NVARCHAR)" == "this is foo",
));

test_type!(str<String>(Mssql,
    "'this is foo'" == "this is foo",
    "''" == "",
));

test_type!(bool(
    Mssql,
    "CAST(1 as BIT)" == true,
    "CAST(0 as BIT)" == false
));

test_type!(bytes<Vec<u8>>(Mssql,
    "CAST(0xDEADBEEF AS VARBINARY(MAX))"
        == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "CAST(0x AS VARBINARY(MAX))"
        == Vec::<u8>::new(),
    "CAST(0x0000000000000000 AS VARBINARY(MAX))"
        == vec![0_u8; 8],
));

test_type!(bytes_single<Vec<u8>>(Mssql,
    "CAST(0xFF AS VARBINARY(MAX))" == vec![0xFF_u8],
));

test_type!(bytes_large<Vec<u8>>(Mssql,
    "CAST(REPLICATE(CAST(0xAB AS VARBINARY(MAX)), 10000) AS VARBINARY(MAX))"
        == vec![0xAB_u8; 10000],
));

test_type!(str_nchar<String>(Mssql,
    "CAST('hello' AS NCHAR(5))" == "hello",
));

test_type!(str_varchar<String>(Mssql,
    "CAST('hello varchar' AS VARCHAR(50))" == "hello varchar",
));

test_type!(str_unicode<String>(Mssql,
    "CAST(N'\u{1F600}\u{1F680}\u{2764}' AS NVARCHAR(MAX))" == "\u{1F600}\u{1F680}\u{2764}",
    "CAST(N'\u{4F60}\u{597D}\u{4E16}\u{754C}' AS NVARCHAR(MAX))" == "\u{4F60}\u{597D}\u{4E16}\u{754C}",
));

test_type!(str_nvarchar_max_large<String>(Mssql,
    "REPLICATE(CAST(N'x' AS NVARCHAR(MAX)), 10000)"
        == "x".repeat(10000),
));

test_type!(null_bool<Option<bool>>(Mssql,
    "CAST(NULL AS BIT)" == None::<bool>,
));

test_type!(null_string<Option<String>>(Mssql,
    "CAST(NULL AS NVARCHAR(100))" == None::<String>,
));

test_type!(null_i64<Option<i64>>(Mssql,
    "CAST(NULL AS BIGINT)" == None::<i64>,
));

test_type!(null_f64<Option<f64>>(Mssql,
    "CAST(NULL AS FLOAT)" == None::<f64>,
));

test_type!(null_bytes<Option<Vec<u8>>>(Mssql,
    "CAST(NULL AS VARBINARY(MAX))" == None::<Vec<u8>>,
));

test_type!(xml<sqlx::mssql::MssqlXml>(Mssql,
    "CAST('<root><item>hello</item></root>' AS XML)"
        == sqlx::mssql::MssqlXml::from("<root><item>hello</item></root>".to_owned()),
));

#[cfg(feature = "uuid")]
test_type!(uuid<sqlx::types::Uuid>(Mssql,
    "CAST('00000000-0000-0000-0000-000000000000' AS UNIQUEIDENTIFIER)"
        == sqlx::types::Uuid::nil(),
    "CAST('936da01f-9abd-4d9d-80c7-02af85c822a8' AS UNIQUEIDENTIFIER)"
        == sqlx::types::Uuid::parse_str("936DA01F-9ABD-4D9D-80C7-02AF85C822A8").unwrap(),
));

#[cfg(feature = "chrono")]
mod chrono {
    use sqlx::mssql::Mssql;
    use sqlx_test::test_type;

    type NaiveDate = sqlx::types::chrono::NaiveDate;
    type NaiveTime = sqlx::types::chrono::NaiveTime;
    type NaiveDateTime = sqlx::types::chrono::NaiveDateTime;
    type DateTimeUtc = sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>;
    type DateTimeFixed = sqlx::types::chrono::DateTime<sqlx::types::chrono::FixedOffset>;
    type FixedOffset = sqlx::types::chrono::FixedOffset;

    test_type!(chrono_naive_date<NaiveDate>(Mssql,
        "CAST('2001-01-05' AS DATE)"
            == NaiveDate::from_ymd_opt(2001, 1, 5).unwrap(),
        "CAST('2050-11-23' AS DATE)"
            == NaiveDate::from_ymd_opt(2050, 11, 23).unwrap(),
    ));

    test_type!(chrono_naive_time<NaiveTime>(Mssql,
        "CAST('05:10:20' AS TIME)"
            == NaiveTime::from_hms_opt(5, 10, 20).unwrap(),
        "CAST('00:00:00' AS TIME)"
            == NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    ));

    test_type!(chrono_naive_date_time<NaiveDateTime>(Mssql,
        "CAST('2019-01-02 05:10:20' AS DATETIME2)"
            == NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2019, 1, 2).unwrap(),
                NaiveTime::from_hms_opt(5, 10, 20).unwrap(),
            ),
    ));

    test_type!(chrono_date_time_utc<DateTimeUtc>(Mssql,
        "CAST('2019-01-02 05:10:20.000 +00:00' AS DATETIMEOFFSET)"
            == NaiveDate::from_ymd_opt(2019, 1, 2)
                .unwrap()
                .and_hms_opt(5, 10, 20)
                .unwrap()
                .and_utc(),
    ));

    test_type!(chrono_date_time_fixed_utc<DateTimeFixed>(Mssql,
        "CAST('2019-01-02 05:10:20.000 +00:00' AS DATETIMEOFFSET)"
            == NaiveDate::from_ymd_opt(2019, 1, 2)
                .unwrap()
                .and_hms_opt(5, 10, 20)
                .unwrap()
                .and_local_timezone(FixedOffset::east_opt(0).unwrap())
                .unwrap(),
    ));

    test_type!(chrono_date_time_fixed_positive<DateTimeFixed>(Mssql,
        "CAST('2024-06-15 14:30:00.000 +05:30' AS DATETIMEOFFSET)"
            == NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(14, 30, 0)
                .unwrap()
                .and_local_timezone(FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap())
                .unwrap(),
    ));

    test_type!(chrono_date_time_fixed_negative<DateTimeFixed>(Mssql,
        "CAST('2024-12-25 08:00:00.000 -08:00' AS DATETIMEOFFSET)"
            == NaiveDate::from_ymd_opt(2024, 12, 25)
                .unwrap()
                .and_hms_opt(8, 0, 0)
                .unwrap()
                .and_local_timezone(FixedOffset::west_opt(8 * 3600).unwrap())
                .unwrap(),
    ));

    // Verify DateTime<Utc> can decode from DATETIMEOFFSET with non-zero offset
    // (the value should be converted to UTC)
    test_type!(chrono_date_time_utc_from_offset<DateTimeUtc>(Mssql,
        "CAST('2024-06-15 14:30:00.000 +05:30' AS DATETIMEOFFSET)"
            == NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(9, 0, 0)
                .unwrap()
                .and_utc(),
    ));
}

#[cfg(feature = "time")]
mod time_tests {
    use sqlx::mssql::Mssql;
    use sqlx_test::test_type;

    type TimeDate = sqlx::types::time::Date;
    type TimeTime = sqlx::types::time::Time;
    type TimePrimitiveDateTime = sqlx::types::time::PrimitiveDateTime;
    type TimeOffsetDateTime = sqlx::types::time::OffsetDateTime;

    use time::macros::{date, datetime, time as time_macro};

    test_type!(time_date<TimeDate>(Mssql,
        "CAST('2001-01-05' AS DATE)"
            == date!(2001-01-05),
        "CAST('2050-11-23' AS DATE)"
            == date!(2050-11-23),
    ));

    test_type!(time_time<TimeTime>(Mssql,
        "CAST('05:10:20' AS TIME)"
            == time_macro!(05:10:20),
        "CAST('00:00:00' AS TIME)"
            == time_macro!(00:00:00),
    ));

    test_type!(time_primitive_date_time<TimePrimitiveDateTime>(Mssql,
        "CAST('2019-01-02 05:10:20' AS DATETIME2)"
            == datetime!(2019-01-02 05:10:20),
    ));

    test_type!(time_offset_date_time<TimeOffsetDateTime>(Mssql,
        "CAST('2019-01-02 05:10:20.000 +00:00' AS DATETIMEOFFSET)"
            == datetime!(2019-01-02 05:10:20 UTC),
    ));
}

#[cfg(feature = "rust_decimal")]
test_type!(rust_decimal<sqlx::types::Decimal>(Mssql,
    "CAST('0' AS DECIMAL(10,2))" == sqlx::types::Decimal::ZERO,
    "CAST('1.23' AS DECIMAL(10,2))" == sqlx::types::Decimal::new(123, 2),
    "CAST('-1.23' AS DECIMAL(10,2))" == sqlx::types::Decimal::new(-123, 2),
));

#[cfg(feature = "rust_decimal")]
test_type!(rust_decimal_money<sqlx::types::Decimal>(Mssql,
    "CAST(1234.5678 AS MONEY)" == sqlx::types::Decimal::new(12345678, 4),
    "CAST(0 AS MONEY)" == sqlx::types::Decimal::ZERO,
));

#[cfg(feature = "bigdecimal")]
test_type!(bigdecimal<sqlx::types::BigDecimal>(Mssql,
    "CAST('0' AS DECIMAL(10,2))" == "0.00".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST('1.23' AS DECIMAL(10,2))" == "1.23".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST('-1.23' AS DECIMAL(10,2))" == "-1.23".parse::<sqlx::types::BigDecimal>().unwrap(),
));

#[cfg(feature = "bigdecimal")]
test_type!(bigdecimal_money<sqlx::types::BigDecimal>(Mssql,
    "CAST(1234.5678 AS MONEY)" == "1234.5678".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(0 AS MONEY)" == "0".parse::<sqlx::types::BigDecimal>().unwrap(),
));

#[cfg(feature = "json")]
mod json_tests {
    use sqlx::mssql::Mssql;
    use sqlx::types::Json;
    use sqlx_test::test_type;

    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct Friend {
        name: String,
        age: u32,
    }

    test_type!(json<Json<Friend>>(Mssql,
        "CAST('{\"name\":\"Joe\",\"age\":33}' AS NVARCHAR(MAX))"
            == Json(Friend { name: "Joe".to_string(), age: 33 }),
    ));

    test_type!(json_value<sqlx::types::JsonValue>(Mssql,
        "CAST('null' AS NVARCHAR(MAX))" == serde_json::Value::Null,
    ));
}
