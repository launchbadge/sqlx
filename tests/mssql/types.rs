use sqlx_oldapi::mssql::Mssql;
use sqlx_test::test_type;

test_type!(str<String>(Mssql,
    "'this is foo'" == "this is foo",
    "''" == "",
    "CAST('foo' AS VARCHAR(3))" == "foo",
    "CAST('foo' AS VARCHAR(max))" == "foo",
    "REPLICATE('a', 910)" == "a".repeat(910),
));

test_type!(str_unicode<String>(Mssql, "N'￮'" == "￮"));

test_type!(long_str<String>(Mssql,
    "REPLICATE(CAST('a' AS VARCHAR), 8000)" == "a".repeat(8000),
    "REPLICATE(CAST('a' AS VARCHAR(max)), 8192)" == "a".repeat(8192),
    "REPLICATE(CAST('a' AS NVARCHAR(max)), 8192)" == "a".repeat(8192),
    "REPLICATE(CAST('a' AS VARCHAR(max)), 100000)" == "a".repeat(100_000),
));

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

test_type!(bool(
    Mssql,
    "CAST(1 as BIT)" == true,
    "CAST(0 as BIT)" == false
));

test_type!(bytes<Vec<u8>>(Mssql,
    "0xDEADBEEF" == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "CAST(' ' AS VARBINARY)" == vec![0x20_u8],
    "CAST(REPLICATE(' ', 31) AS VARBINARY(max))" == vec![0x20_u8; 31],
));

test_type!(long_byte_buffer<Vec<u8>>(Mssql,
    "CAST(REPLICATE(CAST(' ' AS VARCHAR(max)), 100000) AS VARBINARY(max))" == vec![0x20_u8; 100000],
));

test_type!(empty_varbinary<Vec<u8>>(Mssql,
    "CAST('' AS VARBINARY)" == Vec::<u8>::new(),
));

test_type!(null_varbinary<Option<Vec<u8>>>(Mssql,
    "CAST(NULL AS VARBINARY)" == None::<Vec<u8>>,
    "CAST(NULL AS VARBINARY(max))" == None::<Vec<u8>>,
));

#[cfg(feature = "chrono")]
mod chrono {
    use super::*;
    use sqlx_oldapi::types::chrono::{DateTime, NaiveDate, NaiveDateTime};

    test_type!(NaiveDateTime(
        Mssql,
        "CAST('2016-10-23 12:45:37.1234567' as DateTime2)"
            == NaiveDateTime::parse_from_str("2016-10-23 12:45:37.1234567", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap()
    ));

    test_type!(DateTime<_>(
        Mssql,
        "CAST('2016-10-23 12:45:37.1234567 +02:00' as datetimeoffset(7))" == DateTime::parse_from_rfc3339("2016-10-23T12:45:37.1234567+02:00").unwrap()
    ));

    test_type!(NaiveDate(
        Mssql,
        "CAST('1789-07-14' AS DATE)"
            == NaiveDate::parse_from_str("1789-07-14", "%Y-%m-%d").unwrap()
    ));
}
