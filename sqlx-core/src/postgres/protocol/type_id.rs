use crate::postgres::types::try_resolve_type_name;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeId(pub(crate) u32);

// DEVELOPER PRO TIP: find builtin type OIDs easily by grepping this file
// https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat
//
// If you have Postgres running locally you can also try
// SELECT oid, typarray FROM pg_type where typname = '<type name>'

#[allow(dead_code)]
impl TypeId {
    // Scalar

    pub(crate) const BOOL: TypeId = TypeId(16);

    pub(crate) const CHAR: TypeId = TypeId(18);

    pub(crate) const INT2: TypeId = TypeId(21);
    pub(crate) const INT4: TypeId = TypeId(23);
    pub(crate) const INT8: TypeId = TypeId(20);

    pub(crate) const OID: TypeId = TypeId(26);

    pub(crate) const FLOAT4: TypeId = TypeId(700);
    pub(crate) const FLOAT8: TypeId = TypeId(701);

    pub(crate) const NUMERIC: TypeId = TypeId(1700);

    pub(crate) const TEXT: TypeId = TypeId(25);
    pub(crate) const VARCHAR: TypeId = TypeId(1043);
    pub(crate) const BPCHAR: TypeId = TypeId(1042);
    pub(crate) const NAME: TypeId = TypeId(19);
    pub(crate) const UNKNOWN: TypeId = TypeId(705);

    pub(crate) const DATE: TypeId = TypeId(1082);
    pub(crate) const TIME: TypeId = TypeId(1083);
    pub(crate) const TIMESTAMP: TypeId = TypeId(1114);
    pub(crate) const TIMESTAMPTZ: TypeId = TypeId(1184);

    pub(crate) const BYTEA: TypeId = TypeId(17);

    pub(crate) const UUID: TypeId = TypeId(2950);

    pub(crate) const CIDR: TypeId = TypeId(650);
    pub(crate) const INET: TypeId = TypeId(869);

    // Arrays

    pub(crate) const ARRAY_BOOL: TypeId = TypeId(1000);

    pub(crate) const ARRAY_CHAR: TypeId = TypeId(1002);

    pub(crate) const ARRAY_INT2: TypeId = TypeId(1005);
    pub(crate) const ARRAY_INT4: TypeId = TypeId(1007);
    pub(crate) const ARRAY_INT8: TypeId = TypeId(1016);

    pub(crate) const ARRAY_OID: TypeId = TypeId(1028);

    pub(crate) const ARRAY_FLOAT4: TypeId = TypeId(1021);
    pub(crate) const ARRAY_FLOAT8: TypeId = TypeId(1022);

    pub(crate) const ARRAY_TEXT: TypeId = TypeId(1009);
    pub(crate) const ARRAY_VARCHAR: TypeId = TypeId(1015);
    pub(crate) const ARRAY_BPCHAR: TypeId = TypeId(1014);
    pub(crate) const ARRAY_NAME: TypeId = TypeId(1003);

    pub(crate) const ARRAY_NUMERIC: TypeId = TypeId(1231);

    pub(crate) const ARRAY_DATE: TypeId = TypeId(1182);
    pub(crate) const ARRAY_TIME: TypeId = TypeId(1183);
    pub(crate) const ARRAY_TIMESTAMP: TypeId = TypeId(1115);
    pub(crate) const ARRAY_TIMESTAMPTZ: TypeId = TypeId(1185);

    pub(crate) const ARRAY_BYTEA: TypeId = TypeId(1001);

    pub(crate) const ARRAY_UUID: TypeId = TypeId(2951);

    pub(crate) const ARRAY_CIDR: TypeId = TypeId(651);
    pub(crate) const ARRAY_INET: TypeId = TypeId(1041);

    // JSON

    pub(crate) const JSON: TypeId = TypeId(114);
    pub(crate) const JSONB: TypeId = TypeId(3802);

    // Records

    pub(crate) const RECORD: TypeId = TypeId(2249);
    pub(crate) const ARRAY_RECORD: TypeId = TypeId(2287);
}

impl Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = try_resolve_type_name(self.0) {
            f.write_str(name)
        } else {
            write!(f, "<{}>", self.0)
        }
    }
}
