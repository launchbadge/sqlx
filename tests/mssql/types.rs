use chrono::{DateTime, NaiveDateTime};
use sqlx::mssql::Mssql;
use sqlx_test::test_type;

test_type!(null<Option<i32>>(Mssql,
    "CAST(NULL as INT)" == None::<i32>
));

test_type!(i8(
    Mssql,
    "CAST(5 AS TINYINT)" == 5_i8,
    "CAST(0 AS TINYINT)" == 0_i8
));

test_type!(i16(Mssql, "CAST(21415 AS SMALLINT)" == 21415_i16));

test_type!(i32(Mssql, "CAST(2141512 AS INT)" == 2141512_i32));

test_type!(i64(Mssql, "CAST(32324324432 AS BIGINT)" == 32324324432_i64));

test_type!(f32(
    Mssql,
    "CAST(3.1410000324249268 AS REAL)" == 3.141f32 as f64 as f32
));

test_type!(f64(
    Mssql,
    "CAST(939399419.1225182 AS FLOAT)" == 939399419.1225182_f64
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

test_type!(NaiveDateTime(
    Mssql,
    "CAST('2016-10-23 12:45:37.1234567' as DateTime2)"
        == NaiveDateTime::from_timestamp(1477226737, 123456700)
));

test_type!(DateTime<_>(
    Mssql,
    "CAST('2016-10-23 12:45:37.1234567 +02:00' as datetimeoffset(7))" == DateTime::parse_from_rfc3339("2016-10-23T12:45:37.1234567+02:00").unwrap()
));
