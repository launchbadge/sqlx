use sqlx::decode::Decode;
use sqlx::encode::Encode;

#[derive(PartialEq, Debug, Encode, Decode)]
struct Foo(i32);

#[test]
#[cfg(feature = "postgres")]
fn encode_with_postgres() {
    use sqlx_core::postgres::Postgres;

    let example = Foo(0x1122_3344);

    let mut encoded = Vec::new();
    let mut encoded_orig = Vec::new();

    Encode::<Postgres>::encode(&example, &mut encoded);
    Encode::<Postgres>::encode(&example.0, &mut encoded_orig);

    assert_eq!(encoded, encoded_orig);
}

#[test]
#[cfg(feature = "mysql")]
fn encode_with_mysql() {
    use sqlx_core::mysql::MySql;

    let example = Foo(0x1122_3344);

    let mut encoded = Vec::new();
    let mut encoded_orig = Vec::new();

    Encode::<MySql>::encode(&example, &mut encoded);
    Encode::<MySql>::encode(&example.0, &mut encoded_orig);

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
