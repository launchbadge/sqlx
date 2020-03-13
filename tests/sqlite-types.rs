use sqlx::Sqlite;
use sqlx_test::test_type;

test_type!(null(
    Sqlite,
    Option<i32>,
    "NULL" == None::<i32>
));

test_type!(bool(
    Sqlite,
    bool,
    "false::boolean" == false,
    "true::boolean" == true
));
