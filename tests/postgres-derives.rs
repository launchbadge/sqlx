use sqlx::{postgres::PgQueryAs, Connection, Cursor, Executor, FromRow, Postgres};
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

// "Strong" enums can map to TEXT (25)
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename = "text")]
#[sqlx(rename_all = "lowercase")]
enum Strong {
    One,
    Two,

    #[sqlx(rename = "four")]
    Three,
}

// rename_all variants
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
enum ColorLower {
    Red,
    Green,
    Blue,
}
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
enum ColorSnake {
    RedGreen,
    BlueBlack,
}
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename_all = "uppercase")]
enum ColorUpper {
    Red,
    Green,
    Blue,
}

// "Strong" enum can map to a custom type
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename = "mood")]
#[sqlx(rename_all = "lowercase")]
enum Mood {
    Ok,
    Happy,
    Sad,
}

// Records must map to a custom type
// Note that all types are types in Postgres
#[derive(PartialEq, Debug, sqlx::Type)]
#[sqlx(rename = "inventory_item")]
struct InventoryItem {
    name: String,
    supplier_id: Option<i32>,
    price: Option<i64>,
}

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

test_type!(strong_color_lower_enum(
    Postgres,
    ColorLower,
    "'green'" == ColorLower::Green
));
test_type!(strong_color_snake_enum(
    Postgres,
    ColorSnake,
    "'red_green'" == ColorSnake::RedGreen
));
test_type!(strong_color_upper_enum(
    Postgres,
    ColorUpper,
    "'GREEN'" == ColorUpper::Green
));

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_enum_type() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute(
        r#"

DROP TABLE IF EXISTS people;

DROP TYPE IF EXISTS mood CASCADE;

CREATE TYPE mood AS ENUM ( 'ok', 'happy', 'sad' );

CREATE TABLE people (
    id      serial PRIMARY KEY,
    mood    mood not null
);
    "#,
    )
    .await?;

    // Drop and re-acquire the connection
    conn.close().await?;
    let mut conn = new::<Postgres>().await?;

    // Select from table test
    let (people_id,): (i32,) = sqlx::query_as(
        "
INSERT INTO people (mood)
VALUES ($1)
RETURNING id
        ",
    )
    .bind(Mood::Sad)
    .fetch_one(&mut conn)
    .await?;

    // Drop and re-acquire the connection
    conn.close().await?;
    let mut conn = new::<Postgres>().await?;

    #[derive(sqlx::FromRow)]
    struct PeopleRow {
        id: i32,
        mood: Mood,
    }

    let rec: PeopleRow = sqlx::query_as(
        "
SELECT id, mood FROM people WHERE id = $1
        ",
    )
    .bind(people_id)
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(rec.id, people_id);
    assert_eq!(rec.mood, Mood::Sad);

    // Drop and re-acquire the connection
    conn.close().await?;
    let mut conn = new::<Postgres>().await?;

    let stmt = format!("SELECT id, mood FROM people WHERE id = {}", people_id);
    dbg!(&stmt);
    let mut cursor = conn.fetch(&*stmt);

    let row = cursor.next().await?.unwrap();
    let rec = PeopleRow::from_row(&row)?;

    assert_eq!(rec.id, people_id);
    assert_eq!(rec.mood, Mood::Sad);

    // Normal type equivalency test

    let rec: (bool, Mood) = sqlx::query_as(
        "
SELECT $1 = 'happy'::mood, $1
        ",
    )
    .bind(&Mood::Happy)
    .fetch_one(&mut conn)
    .await?;

    assert!(rec.0);
    assert_eq!(rec.1, Mood::Happy);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_record_type() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute(
        r#"
DO $$ BEGIN

CREATE TYPE inventory_item AS (
    name            text,
    supplier_id     int,
    price           bigint
);

EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
    "#,
    )
    .await?;

    let value = InventoryItem {
        name: "fuzzy dice".to_owned(),
        supplier_id: Some(42),
        price: Some(199),
    };

    let rec: (bool, InventoryItem) = sqlx::query_as(
        "
        SELECT $1 = ROW('fuzzy dice', 42, 199)::inventory_item, $1
        ",
    )
    .bind(&value)
    .fetch_one(&mut conn)
    .await?;

    assert!(rec.0);
    assert_eq!(rec.1, value);

    Ok(())
}

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

#[cfg(feature = "macros")]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_from_row_with_keyword() -> anyhow::Result<()> {
    use sqlx::prelude::*;

    #[derive(Debug, sqlx::FromRow)]
    struct AccountKeyword {
        r#type: i32,
        r#static: String,
        r#let: Option<String>,
        r#struct: Option<String>,
        name: Option<String>,
    }

    let mut conn = new::<Postgres>().await?;

    let account: AccountKeyword = sqlx::query_as(
        r#"SELECT * from (VALUES (1, 'foo', 'bar', null, null)) accounts(type, static, let, struct, name)"#
    )
    .fetch_one(&mut conn)
    .await?;
    println!("{:?}", account);

    assert_eq!(1, account.r#type);
    assert_eq!("foo", account.r#static);
    assert_eq!(None, account.r#struct);
    assert_eq!(Some("bar".to_owned()), account.r#let);
    assert_eq!(None, account.name);

    Ok(())
}

#[cfg(feature = "macros")]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_from_row_with_rename() -> anyhow::Result<()> {
    use sqlx::prelude::*;

    #[derive(Debug, sqlx::FromRow)]
    struct AccountKeyword {
        #[sqlx(rename = "type")]
        own_type: i32,

        #[sqlx(rename = "static")]
        my_static: String,

        #[sqlx(rename = "let")]
        custom_let: Option<String>,

        #[sqlx(rename = "struct")]
        def_struct: Option<String>,

        name: Option<String>,
    }

    let mut conn = new::<Postgres>().await?;

    let account: AccountKeyword = sqlx::query_as(
        r#"SELECT * from (VALUES (1, 'foo', 'bar', null, null)) accounts(type, static, let, struct, name)"#
    )
    .fetch_one(&mut conn)
    .await?;
    println!("{:?}", account);

    assert_eq!(1, account.own_type);
    assert_eq!("foo", account.my_static);
    assert_eq!(None, account.def_struct);
    assert_eq!(Some("bar".to_owned()), account.custom_let);
    assert_eq!(None, account.name);

    Ok(())
}
