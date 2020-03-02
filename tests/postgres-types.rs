use sqlx::Postgres;
use sqlx_test::test_type;

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

#[cfg(feature = "uuid")]
test_type!(uuid(
    Postgres,
    sqlx::types::Uuid,
    "'b731678f-636f-4135-bc6f-19440c13bd19'::uuid"
        == sqlx::types::Uuid::parse_str("b731678f-636f-4135-bc6f-19440c13bd19").unwrap(),
    "'00000000-0000-0000-0000-000000000000'::uuid"
        == sqlx::types::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
));

#[cfg(feature = "chrono")]
mod chrono {
    use super::*;
    use sqlx::types::chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

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
        "'2019-01-02 05:10:20'" == NaiveDate::from_ymd(2019, 1, 2).and_hms(5, 10, 20)
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
