use sqlx::decode::Decode;
use sqlx::encode::Encode;

#[derive(PartialEq, Debug, Encode, Decode)]
struct Foo(i32);

#[test]
#[cfg(feature = "mysql")]
fn encode_mysql() {
    encode_with_db::<sqlx::MySql>();
}

#[test]
#[cfg(feature = "postgres")]
fn encode_postgres() {
    encode_with_db::<sqlx::Postgres>();
}

#[allow(dead_code)]
fn encode_with_db<DB: sqlx::Database>()
where
    Foo: Encode<DB>,
    i32: Encode<DB>,
{
    let example = Foo(0x1122_3344);

    let mut encoded = Vec::new();
    let mut encoded_orig = Vec::new();

    Encode::<DB>::encode(&example, &mut encoded);
    Encode::<DB>::encode(&example.0, &mut encoded_orig);

    assert_eq!(encoded, encoded_orig);
}

#[test]
#[cfg(feature = "mysql")]
fn decode_mysql() {
    decode_with_db();
}

#[test]
#[cfg(feature = "postgres")]
fn decode_postgres() {
    decode_with_db();
}

#[cfg(feature = "postgres")]
fn decode_with_db()
where
    Foo: for<'de> Decode<'de, sqlx::Postgres> + Encode<sqlx::Postgres>,
{
    let example = Foo(0x1122_3344);

    let mut encoded = Vec::new();
    Encode::<sqlx::Postgres>::encode(&example, &mut encoded);

    let decoded = Foo::decode(Some(sqlx::postgres::PgValue::Binary(&encoded))).unwrap();
    assert_eq!(example, decoded);
}

#[cfg(feature = "mysql")]
fn decode_with_db()
where
    Foo: for<'de> Decode<'de, sqlx::MySql> + Encode<sqlx::MySql>,
{
    let example = Foo(0x1122_3344);

    let mut encoded = Vec::new();
    Encode::<sqlx::MySql>::encode(&example, &mut encoded);

    let decoded = Foo::decode(Some(sqlx::mysql::MySqlValue::Binary(&encoded))).unwrap();
    assert_eq!(example, decoded);
}
