// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/binary__log__types_8h.html
// https://mariadb.com/kb/en/library/resultset/#field-types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeId(pub u8);

// https://github.com/google/mysql/blob/c01fc2134d439282a21a2ddf687566e198ddee28/include/mysql_com.h#L429
impl TypeId {
    pub const NULL: TypeId = TypeId(6);

    // String: CHAR, VARCHAR, TEXT
    // Bytes: BINARY, VARBINARY, BLOB
    pub const CHAR: TypeId = TypeId(254); // or BINARY
    pub const VAR_CHAR: TypeId = TypeId(253); // or VAR_BINARY
    pub const TEXT: TypeId = TypeId(252); // or BLOB

    // Enum
    pub const ENUM: TypeId = TypeId(247);

    // More Bytes
    pub const TINY_BLOB: TypeId = TypeId(249);
    pub const MEDIUM_BLOB: TypeId = TypeId(250);
    pub const LONG_BLOB: TypeId = TypeId(251);

    // Numeric: TINYINT, SMALLINT, INT, BIGINT
    pub const TINY_INT: TypeId = TypeId(1);
    pub const SMALL_INT: TypeId = TypeId(2);
    pub const INT: TypeId = TypeId(3);
    pub const BIG_INT: TypeId = TypeId(8);
    // pub const MEDIUM_INT: TypeId = TypeId(9);

    // Numeric: FLOAT, DOUBLE
    pub const FLOAT: TypeId = TypeId(4);
    pub const DOUBLE: TypeId = TypeId(5);
    pub const NEWDECIMAL: TypeId = TypeId(246);

    // Date/Time: DATE, TIME, DATETIME, TIMESTAMP
    pub const DATE: TypeId = TypeId(10);
    pub const TIME: TypeId = TypeId(11);
    pub const DATETIME: TypeId = TypeId(12);
    pub const TIMESTAMP: TypeId = TypeId(7);
}

impl Default for TypeId {
    fn default() -> TypeId {
        TypeId::NULL
    }
}
