use crate::Database;

#[derive(Debug)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "DB::TypeInfo: serde::Serialize, DB::Column: serde::Serialize",
        deserialize = "DB::TypeInfo: serde::de::DeserializeOwned, DB::Column: serde::de::DeserializeOwned",
    ))
)]
#[doc(hidden)]
pub struct Describe<Db: Database> {
    pub columns: Vec<Db::Column>,
    pub parameters: Vec<Db::TypeInfo>,
    pub nullable: Vec<Option<bool>>,
}
