use sqlx::Sqlite;
use sqlx_core::impl_into_encode_for_db;
use sqlx_test::test_type;

#[derive(Debug, PartialEq, sqlx::Type)]
#[repr(u32)]
enum Origin {
    Foo = 1,
    Bar = 2,
}

test_type!(origin_enum<Origin>(Sqlite,
    "1" == Origin::Foo,
    "2" == Origin::Bar,
));

impl_into_encode_for_db!(Sqlite, Origin);
