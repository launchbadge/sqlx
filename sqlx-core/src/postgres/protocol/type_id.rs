use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub(crate) const INT2: TypeId = TypeId(21);
    pub(crate) const INT4: TypeId = TypeId(23);
    pub(crate) const INT8: TypeId = TypeId(20);

    pub(crate) const FLOAT4: TypeId = TypeId(700);
    pub(crate) const FLOAT8: TypeId = TypeId(701);

    pub(crate) const NUMERIC: TypeId = TypeId(1700);

    pub(crate) const TEXT: TypeId = TypeId(25);
    pub(crate) const VARCHAR: TypeId = TypeId(1043);
    pub(crate) const BPCHAR: TypeId = TypeId(1042);

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

    pub(crate) const ARRAY_INT2: TypeId = TypeId(1005);
    pub(crate) const ARRAY_INT4: TypeId = TypeId(1007);
    pub(crate) const ARRAY_INT8: TypeId = TypeId(1016);

    pub(crate) const ARRAY_FLOAT4: TypeId = TypeId(1021);
    pub(crate) const ARRAY_FLOAT8: TypeId = TypeId(1022);

    pub(crate) const ARRAY_TEXT: TypeId = TypeId(1009);
    pub(crate) const ARRAY_VARCHAR: TypeId = TypeId(1015);
    pub(crate) const ARRAY_BPCHAR: TypeId = TypeId(1014);

    pub(crate) const ARRAY_NUMERIC: TypeId = TypeId(1700);

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
}

impl Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TypeId::BOOL => f.write_str("BOOL"),

            TypeId::INT2 => f.write_str("INT2"),
            TypeId::INT4 => f.write_str("INT4"),
            TypeId::INT8 => f.write_str("INT8"),

            TypeId::FLOAT4 => f.write_str("FLOAT4"),
            TypeId::FLOAT8 => f.write_str("FLOAT8"),

            TypeId::NUMERIC => f.write_str("NUMERIC"),

            TypeId::TEXT => f.write_str("TEXT"),
            TypeId::VARCHAR => f.write_str("VARCHAR"),
            TypeId::BPCHAR => f.write_str("BPCHAR"),

            TypeId::DATE => f.write_str("DATE"),
            TypeId::TIME => f.write_str("TIME"),
            TypeId::TIMESTAMP => f.write_str("TIMESTAMP"),
            TypeId::TIMESTAMPTZ => f.write_str("TIMESTAMPTZ"),

            TypeId::BYTEA => f.write_str("BYTEA"),

            TypeId::UUID => f.write_str("UUID"),

            TypeId::CIDR => f.write_str("CIDR"),
            TypeId::INET => f.write_str("INET"),

            TypeId::ARRAY_BOOL => f.write_str("BOOL[]"),

            TypeId::ARRAY_INT2 => f.write_str("INT2[]"),
            TypeId::ARRAY_INT4 => f.write_str("INT4[]"),
            TypeId::ARRAY_INT8 => f.write_str("INT8[]"),

            TypeId::ARRAY_FLOAT4 => f.write_str("FLOAT4[]"),
            TypeId::ARRAY_FLOAT8 => f.write_str("FLOAT8[]"),

            TypeId::ARRAY_TEXT => f.write_str("TEXT[]"),
            TypeId::ARRAY_VARCHAR => f.write_str("VARCHAR[]"),
            TypeId::ARRAY_BPCHAR => f.write_str("BPCHAR[]"),

            TypeId::ARRAY_NUMERIC => f.write_str("NUMERIC[]"),

            TypeId::ARRAY_DATE => f.write_str("DATE[]"),
            TypeId::ARRAY_TIME => f.write_str("TIME[]"),
            TypeId::ARRAY_TIMESTAMP => f.write_str("TIMESTAMP[]"),
            TypeId::ARRAY_TIMESTAMPTZ => f.write_str("TIMESTAMPTZ[]"),

            TypeId::ARRAY_BYTEA => f.write_str("BYTEA[]"),

            TypeId::ARRAY_UUID => f.write_str("UUID[]"),

            TypeId::ARRAY_CIDR => f.write_str("CIDR[]"),
            TypeId::ARRAY_INET => f.write_str("INET[]"),

            TypeId::JSON => f.write_str("JSON"),
            TypeId::JSONB => f.write_str("JSONB"),

            _ => write!(f, "<{}>", self.0),
        }
    }
}
