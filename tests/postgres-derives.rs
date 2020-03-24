use sqlx::Postgres;
use sqlx_test::{new, test_type};
use std::fmt::Debug;

// Transparent types are rust-side wrappers over DB types
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct Transparent(i32);

// "Weak" enums map to an integer type indicated by #[repr]
#[derive(PartialEq, Copy, Clone, Debug, sqlx::Type)]
#[repr(i32)]
enum Weak {
    One = 0,
    Two = 2,
    Three = 4,
}

// "Strong" enums can map to TEXT (25) or a custom enum type
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(postgres(oid = 25))]
#[sqlx(rename_all = "lowercase")]
enum Strong {
    One,
    Two,

    #[sqlx(rename = "four")]
    Three,
}

// TODO: Figure out a good solution for custom type testing
// Records must map to a custom type
// Note that all types are types in Postgres
// #[derive(PartialEq, Debug, sqlx::Type)]
// #[sqlx(postgres(oid = ?))]
// struct InventoryItem {
//     name: String,
//     supplier_id: Option<i32>,
//     price: Option<i64>
// }

test_type!(transparent(
    Postgres,
    Transparent,
    "0" == Transparent(0),
    "23523" == Transparent(23523)
));

test_type!(weak_enum(
    Postgres,
    Weak,
    "0::int4" == Weak::One,
    "2::int4" == Weak::Two,
    "4::int4" == Weak::Three
));

test_type!(strong_enum(
    Postgres,
    Strong,
    "'one'::text" == Strong::One,
    "'two'::text" == Strong::Two,
    "'four'::text" == Strong::Three
));

// TODO: Figure out a good solution for custom type testing
// test_type!(record_pg_config(
//     Postgres,
//     InventoryItem,
//     "(SELECT ROW('fuzzy dice', 42, 199)::inventory_item)"
//         == InventoryItem {
//             name: "fuzzy dice".to_owned(),
//             supplier_id: Some(42),
//             price: Some(199),
//         },
//     "(SELECT '(\"fuuzy dice\",,)'::pg_config)"
//         == InventoryItem {
//             name: "fuzzy dice".to_owned(),
//             supplier_id: None,
//             price: None,
//         },
//     "(SELECT '(\"\",,2350)'::pg_config)"
//         == InventoryItem {
//             name: "".to_owned(),
//             supplier_id: None,
//             price: Some(2350)
//         }
// ));

#[cfg(feature = "macros")]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_from_row() -> anyhow::Result<()> {
    // Needed for PgQueryAs
    use sqlx::prelude::*;

    let mut conn = new::<Postgres>().await?;

    #[derive(sqlx::FromRow)]
    struct Account {
        id: i32,
        name: String,
    }

    let account: Account = sqlx::query_as(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
    )
    .bind(1_i32)
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");

    // A _single_ lifetime may be used but only when using the lowest-level API currently (Query::fetch)

    #[derive(sqlx::FromRow)]
    struct RefAccount<'a> {
        id: i32,
        name: &'a str,
    }

    let mut cursor = sqlx::query(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
    )
    .bind(1_i32)
    .fetch(&mut conn);

    let account = RefAccount::from_row(&cursor.next().await?.unwrap())?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");

    Ok(())
}
