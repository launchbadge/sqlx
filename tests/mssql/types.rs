use sqlx::mssql::MsSql;
use sqlx_test::test_type;

test_type!(i8(
    MsSql,
    "CAST(5 AS TINYINT)" == 5_i8,
    "CAST(0 AS TINYINT)" == 0_i8
));

test_type!(i16(MsSql, "CAST(21415 AS SMALLINT)" == 21415_i16));

test_type!(i32(MsSql, "CAST(2141512 AS INT)" == 2141512_i32));

test_type!(i64(MsSql, "CAST(32324324432 AS BIGINT)" == 32324324432_i64));
