use crate::decode::Decode;
use crate::encode::Encode;
use crate::mysql::database::MySql;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::*;
use crate::mysql::{MySqlTypeInfo, MySqlValue};
use crate::types::{Json, Type};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

impl Type<MySql> for JsonValue {
    fn type_info() -> MySqlTypeInfo {
        <Json<Self> as Type<MySql>>::type_info()
    }
}

impl<T> Type<MySql> for Json<T> {
    fn type_info() -> MySqlTypeInfo {
        // MySql uses the CHAR type to pass JSON data from and to the client
        MySqlTypeInfo::new(TypeId::CHAR)
    }
}

impl<T> Encode<MySql> for Json<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        let json_string_value =
            serde_json::to_string(&self.0).expect("serde_json failed to convert to string");
        <str as Encode<MySql>>::encode(json_string_value.as_str(), buf);
    }
}

impl<'de, T> Decode<'de, MySql> for Json<T>
where
    T: 'de,
    T: for<'de1> Deserialize<'de1>,
{
    fn decode(value: MySqlValue<'de>) -> crate::Result<Self> {
        let string_value = <&'de str as Decode<MySql>>::decode(value).unwrap();
        serde_json::from_str(&string_value)
            .map(Json)
            .map_err(crate::Error::decode)
    }
}
