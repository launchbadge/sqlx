use sqlx::decode::Decode;
use sqlx::encode::Encode;
use sqlx::types::{HasSqlType, TypeInfo};
use std::fmt::Debug;

#[derive(PartialEq, Debug, Encode, Decode, HasSqlType)]
#[sqlx(transparent)]
struct Transparent(i32);

#[derive(PartialEq, Debug, Clone, Copy, Encode, Decode, HasSqlType)]
#[repr(i32)]
#[allow(dead_code)]
enum Weak {
    One,
    Two,
    Three,
}

#[derive(PartialEq, Debug, Encode, Decode, HasSqlType)]
#[sqlx(postgres(oid = 10101010))]
#[allow(dead_code)]
enum Strong {
    One,
    Two,
    #[sqlx(rename = "four")]
    Three,
}

#[derive(PartialEq, Debug, Encode, Decode, HasSqlType)]
#[sqlx(postgres(oid = 20202020))]
#[allow(dead_code)]
struct Struct {
    field1: String,
    field2: i64,
    field3: bool,
}

#[test]
#[cfg(feature = "mysql")]
fn encode_transparent_mysql() {
    encode_transparent::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn encode_transparent_postgres() {
    encode_transparent::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn encode_transparent<DB: sqlx::Database>()
where
    Transparent: Encode<DB>,
    i32: Encode<DB>,
{
    let example = Transparent(0x1122_3344);

    let mut encoded = Vec::new();
    let mut encoded_orig = Vec::new();

    Encode::<DB>::encode(&example, &mut encoded);
    Encode::<DB>::encode(&example.0, &mut encoded_orig);

    assert_eq!(encoded, encoded_orig);
}

#[test]
#[cfg(feature = "mysql")]
fn encode_weak_enum_mysql() {
    encode_weak_enum::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn encode_weak_enum_postgres() {
    encode_weak_enum::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn encode_weak_enum<DB: sqlx::Database>()
where
    Weak: Encode<DB>,
    i32: Encode<DB>,
{
    for example in [Weak::One, Weak::Two, Weak::Three].iter() {
        let mut encoded = Vec::new();
        let mut encoded_orig = Vec::new();

        Encode::<DB>::encode(example, &mut encoded);
        Encode::<DB>::encode(&(*example as i32), &mut encoded_orig);

        assert_eq!(encoded, encoded_orig);
    }
}

#[test]
#[cfg(feature = "mysql")]
fn encode_strong_enum_mysql() {
    encode_strong_enum::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn encode_strong_enum_postgres() {
    encode_strong_enum::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn encode_strong_enum<DB: sqlx::Database>()
where
    Strong: Encode<DB>,
    str: Encode<DB>,
{
    for (example, name) in [
        (Strong::One, "One"),
        (Strong::Two, "Two"),
        (Strong::Three, "four"),
    ]
    .iter()
    {
        let mut encoded = Vec::new();
        let mut encoded_orig = Vec::new();

        Encode::<DB>::encode(example, &mut encoded);
        Encode::<DB>::encode(*name, &mut encoded_orig);

        assert_eq!(encoded, encoded_orig);
    }
}

#[test]
#[cfg(feature = "postgres")]
fn encode_struct_postgres() {
    let field1 = "Foo".to_string();
    let field2 = 3;
    let field3 = false;

    let example = Struct {
        field1: field1.clone(),
        field2,
        field3,
    };

    let mut encoded = Vec::new();
    Encode::<sqlx::Postgres>::encode(&example, &mut encoded);

    let string_oid = <sqlx::Postgres as HasSqlType<String>>::type_info().oid();
    let i64_oid = <sqlx::Postgres as HasSqlType<i64>>::type_info().oid();
    let bool_oid = <sqlx::Postgres as HasSqlType<bool>>::type_info().oid();

    // 3 columns
    assert_eq!(&[0, 0, 0, 3], &encoded[..4]);
    let encoded = &encoded[4..];

    // check field1 (string)
    assert_eq!(&string_oid.to_be_bytes(), &encoded[0..4]);
    assert_eq!(&(field1.len() as u32).to_be_bytes(), &encoded[4..8]);
    assert_eq!(field1.as_bytes(), &encoded[8..8 + field1.len()]);
    let encoded = &encoded[8 + field1.len()..];

    // check field2 (i64)
    assert_eq!(&i64_oid.to_be_bytes(), &encoded[0..4]);
    assert_eq!(&8u32.to_be_bytes(), &encoded[4..8]);
    assert_eq!(field2.to_be_bytes(), &encoded[8..16]);
    let encoded = &encoded[16..];

    // check field3 (bool)
    assert_eq!(&bool_oid.to_be_bytes(), &encoded[0..4]);
    assert_eq!(&1u32.to_be_bytes(), &encoded[4..8]);
    assert_eq!(field3, encoded[8] != 0);
    let encoded = &encoded[9..];

    assert!(encoded.is_empty());

    let string_size = <String as Encode<sqlx::Postgres>>::size_hint(&field1);
    let i64_size = <i64 as Encode<sqlx::Postgres>>::size_hint(&field2);
    let bool_size = <bool as Encode<sqlx::Postgres>>::size_hint(&field3);

    assert_eq!(
        4 + 3 * (4 + 4) + string_size + i64_size + bool_size,
        example.size_hint()
    );
}

#[test]
#[cfg(feature = "mysql")]
fn decode_transparent_mysql() {
    decode_with_db::<sqlx::MySql, Transparent>(Transparent(0x1122_3344));
}

#[test]
#[cfg(feature = "postgres")]
fn decode_transparent_postgres() {
    decode_with_db::<sqlx::Postgres, Transparent>(Transparent(0x1122_3344));
}

#[test]
#[cfg(feature = "mysql")]
fn decode_weak_enum_mysql() {
    decode_with_db::<sqlx::MySql, Weak>(Weak::One);
    decode_with_db::<sqlx::MySql, Weak>(Weak::Two);
    decode_with_db::<sqlx::MySql, Weak>(Weak::Three);
}

#[test]
#[cfg(feature = "postgres")]
fn decode_weak_enum_postgres() {
    decode_with_db::<sqlx::Postgres, Weak>(Weak::One);
    decode_with_db::<sqlx::Postgres, Weak>(Weak::Two);
    decode_with_db::<sqlx::Postgres, Weak>(Weak::Three);
}

#[test]
#[cfg(feature = "mysql")]
fn decode_strong_enum_mysql() {
    decode_with_db::<sqlx::MySql, Strong>(Strong::One);
    decode_with_db::<sqlx::MySql, Strong>(Strong::Two);
    decode_with_db::<sqlx::MySql, Strong>(Strong::Three);
}

#[test]
#[cfg(feature = "postgres")]
fn decode_strong_enum_postgres() {
    decode_with_db::<sqlx::Postgres, Strong>(Strong::One);
    decode_with_db::<sqlx::Postgres, Strong>(Strong::Two);
    decode_with_db::<sqlx::Postgres, Strong>(Strong::Three);
}

#[test]
#[cfg(feature = "postgres")]
fn decode_struct_postgres() {
    decode_with_db::<sqlx::Postgres, Struct>(Struct {
        field1: "Foo".to_string(),
        field2: 3,
        field3: true,
    });
}

#[allow(dead_code)]
fn decode_with_db<DB: sqlx::Database, V: Decode<DB> + Encode<DB> + PartialEq + Debug>(example: V) {
    let mut encoded = Vec::new();
    Encode::<DB>::encode(&example, &mut encoded);

    let decoded = V::decode(&encoded).unwrap();
    assert_eq!(example, decoded);
}

#[test]
#[cfg(feature = "mysql")]
fn has_sql_type_transparent_mysql() {
    has_sql_type_transparent::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn has_sql_type_transparent_postgres() {
    has_sql_type_transparent::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn has_sql_type_transparent<DB: sqlx::Database>()
where
    DB: HasSqlType<Transparent> + HasSqlType<i32>,
{
    let info: DB::TypeInfo = <DB as HasSqlType<Transparent>>::type_info();
    let info_orig: DB::TypeInfo = <DB as HasSqlType<i32>>::type_info();
    assert!(info.compatible(&info_orig));
}

#[test]
#[cfg(feature = "mysql")]
fn has_sql_type_weak_enum_mysql() {
    has_sql_type_weak_enum::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn has_sql_type_weak_enum_postgres() {
    has_sql_type_weak_enum::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn has_sql_type_weak_enum<DB: sqlx::Database>()
where
    DB: HasSqlType<Weak> + HasSqlType<i32>,
{
    let info: DB::TypeInfo = <DB as HasSqlType<Weak>>::type_info();
    let info_orig: DB::TypeInfo = <DB as HasSqlType<i32>>::type_info();
    assert!(info.compatible(&info_orig));
}

#[test]
#[cfg(feature = "mysql")]
fn has_sql_type_strong_enum_mysql() {
    let info: sqlx::mysql::MySqlTypeInfo = <sqlx::MySql as HasSqlType<Strong>>::type_info();
    assert!(info.compatible(&sqlx::mysql::MySqlTypeInfo::r#enum()))
}

#[test]
#[cfg(feature = "postgres")]
fn has_sql_type_strong_enum_postgres() {
    let info: sqlx::postgres::PgTypeInfo = <sqlx::Postgres as HasSqlType<Strong>>::type_info();
    assert!(info.compatible(&sqlx::postgres::PgTypeInfo::with_oid(10101010)))
}

#[test]
#[cfg(feature = "postgres")]
fn has_sql_type_struct_postgres() {
    let info: sqlx::postgres::PgTypeInfo = <sqlx::Postgres as HasSqlType<Struct>>::type_info();
    assert!(info.compatible(&sqlx::postgres::PgTypeInfo::with_oid(20202020)))
}
