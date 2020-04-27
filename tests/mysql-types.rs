extern crate time_ as time;

use sqlx::MySql;
use sqlx_test::test_type;

test_type!(null(
    MySql,
    Option<i16>,
    "NULL" == None::<i16>
));

test_type!(bool(MySql, bool, "false" == false, "true" == true));

test_type!(u8(MySql, u8, "CAST(253 AS UNSIGNED)" == 253_u8));
test_type!(i8(MySql, i8, "5" == 5_i8, "0" == 0_i8));

test_type!(u16(MySql, u16, "CAST(21415 AS UNSIGNED)" == 21415_u16));
test_type!(i16(MySql, i16, "21415" == 21415_i16));

test_type!(u32(MySql, u32, "CAST(2141512 AS UNSIGNED)" == 2141512_u32));
test_type!(i32(MySql, i32, "2141512" == 2141512_i32));

test_type!(u64(MySql, u64, "CAST(2141512 AS UNSIGNED)" == 2141512_u64));
test_type!(i64(MySql, i64, "2141512" == 2141512_i64));

test_type!(double(MySql, f64, "3.14159265E0" == 3.14159265f64));

// NOTE: This behavior can be very surprising. MySQL implicitly widens FLOAT bind parameters
//       to DOUBLE. This results in the weirdness you see below. MySQL generally recommends to stay
//       away from FLOATs.
test_type!(float(
    MySql,
    f32,
    "3.1410000324249268e0" == 3.141f32 as f64 as f32
));

test_type!(string(
    MySql,
    String,
    "'helloworld'" == "helloworld",
    "''" == ""
));

test_type!(bytes(
    MySql,
    Vec<u8>,
    "X'DEADBEEF'"
        == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "X''"
        == Vec::<u8>::new(),
    "X'0000000052'"
        == vec![0_u8, 0, 0, 0, 0x52]
));

#[cfg(feature = "chrono")]
mod chrono {
    use super::*;
    use sqlx::types::chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    test_type!(chrono_date(
        MySql,
        NaiveDate,
        "DATE '2001-01-05'" == NaiveDate::from_ymd(2001, 1, 5),
        "DATE '2050-11-23'" == NaiveDate::from_ymd(2050, 11, 23)
    ));

    test_type!(chrono_time(
        MySql,
        NaiveTime,
        "TIME '05:10:20.115100'" == NaiveTime::from_hms_micro(5, 10, 20, 115100)
    ));

    test_type!(chrono_date_time(
        MySql,
        NaiveDateTime,
        "TIMESTAMP '2019-01-02 05:10:20'" == NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20)
    ));

    test_type!(chrono_timestamp(
        MySql,
        DateTime::<Utc>,
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
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
        MySql,
        Date,
        "DATE '2001-01-05'" == date!(2001 - 1 - 5),
        "DATE '2050-11-23'" == date!(2050 - 11 - 23)
    ));

    test_type!(time_time(
        MySql,
        Time,
        "TIME '05:10:20.115100'" == time!(5:10:20.115100)
    ));

    test_type!(time_date_time(
        MySql,
        PrimitiveDateTime,
        "TIMESTAMP '2019-01-02 05:10:20'" == date!(2019 - 1 - 2).with_time(time!(5:10:20)),
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2).with_time(time!(5:10:20.115100))
    ));

    test_type!(time_timestamp(
        MySql,
        OffsetDateTime,
        "TIMESTAMP '2019-01-02 05:10:20.115100'"
            == date!(2019 - 1 - 2)
                .with_time(time!(5:10:20.115100))
                .assume_utc()
    ));
}

#[cfg(feature = "bigdecimal")]
test_type!(decimal(
    MySql,
    sqlx::types::BigDecimal,
    "CAST(0 as DECIMAL(0, 0))" == "0".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(1 AS DECIMAL(1, 0))" == "1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(10000 AS DECIMAL(5, 0))" == "10000".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(0.1 AS DECIMAL(2, 1))" == "0.1".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(0.01234 AS DECIMAL(6, 5))" == "0.01234".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(12.34 AS DECIMAL(4, 2))" == "12.34".parse::<sqlx::types::BigDecimal>().unwrap(),
    "CAST(12345.6789 AS DECIMAL(9, 4))" == "12345.6789".parse::<sqlx::types::BigDecimal>().unwrap(),
));

#[cfg(feature = "json")]
mod json_tests {
    use super::*;
    use serde_json::{json, Value as JsonValue};
    use sqlx::types::Json;
    use sqlx_test::test_type;

    test_type!(json(
        MySql,
        JsonValue,
        "SELECT CAST({0} AS JSON) <=> CAST(? AS JSON), '<UNKNOWN>' as _1, ? as _2, ? as _3",
        "'\"Hello, World\"'" == json!("Hello, World"),
        "'\"😎\"'" == json!("😎"),
        "'\"🙋‍♀️\"'" == json!("🙋‍♀️"),
        "'[\"Hello\", \"World!\"]'" == json!(["Hello", "World!"])
    ));

    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct Friend {
        name: String,
        age: u32,
    }

    test_type!(json_struct(
        MySql,
        Json<Friend>,
        "SELECT CAST({0} AS JSON) <=> CAST(? AS JSON), '<UNKNOWN>' as _1, ? as _2, ? as _3",
        "\'{\"name\": \"Joe\", \"age\":33}\'" == Json(Friend { name: "Joe".to_string(), age: 33 })
    ));
}
