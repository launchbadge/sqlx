use sqlx::encode::Encode;
use sqlx::postgres::{decode_struct_field, encode_struct_field};
use sqlx::types::HasSqlType;
use sqlx::Postgres;
use std::convert::TryInto;

#[test]
fn test_encode_field() {
    let value = "Foo Bar";
    let mut raw_encoded = Vec::new();
    <&str as Encode<Postgres>>::encode(&value, &mut raw_encoded);
    let mut field_encoded = Vec::new();
    encode_struct_field(&mut field_encoded, &value);

    // check oid
    let oid = <Postgres as HasSqlType<&str>>::type_info().oid();
    let field_encoded_oid = u32::from_be_bytes(field_encoded[0..4].try_into().unwrap());
    assert_eq!(oid, field_encoded_oid);

    // check length
    let field_encoded_length = u32::from_be_bytes(field_encoded[4..8].try_into().unwrap());
    assert_eq!(raw_encoded.len(), field_encoded_length as usize);

    // check data
    assert_eq!(raw_encoded, &field_encoded[8..]);
}

#[test]
fn test_decode_field() {
    let value = "Foo Bar".to_string();

    let mut buf = Vec::new();
    encode_struct_field(&mut buf, &value);

    let mut buf = buf.as_slice();
    let value_decoded: String = decode_struct_field(&mut buf).unwrap();
    assert_eq!(value_decoded, value);
    assert!(buf.is_empty());
}
