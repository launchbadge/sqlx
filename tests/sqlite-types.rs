use sqlx::Sqlite;
use sqlx_test::test_type;

test_type!(null(
    Sqlite,
    Option<i32>,
    "NULL" == None::<i32>
));

test_type!(bool(Sqlite, bool, "FALSE" == false, "TRUE" == true));

test_type!(i32(Sqlite, i32, "94101" == 94101_i32));

test_type!(i64(Sqlite, i64, "9358295312" == 9358295312_i64));

// NOTE: This behavior can be surprising. Floating-point parameters are widening to double which can
//       result in strange rounding.
test_type!(f32(
    Sqlite,
    f32,
    "3.1410000324249268" == 3.141f32 as f64 as f32
));

test_type!(f64(
    Sqlite,
    f64,
    "939399419.1225182" == 939399419.1225182_f64
));

test_type!(string(
    Sqlite,
    String,
    "'this is foo'" == "this is foo",
    "''" == ""
));

test_type!(bytes(
    Sqlite,
    Vec<u8>,
    "X'DEADBEEF'"
        == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "X''"
        == Vec::<u8>::new(),
    "X'0000000052'"
        == vec![0_u8, 0, 0, 0, 0x52]
));
