// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/binary__log__types_8h.html
// https://mariadb.com/kb/en/library/resultset/#field-types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Type(pub u8);

impl Type {
    pub const BIT: Type = Type(16);
    pub const BLOB: Type = Type(252);
    pub const DATE: Type = Type(10);
    pub const DATETIME: Type = Type(12);
    pub const DECIMAL: Type = Type(0);
    pub const DOUBLE: Type = Type(5);
    pub const ENUM: Type = Type(247);
    pub const FLOAT: Type = Type(4);
    pub const GEOMETRY: Type = Type(255);
    pub const INT24: Type = Type(9);
    pub const JSON: Type = Type(245); // MySQL Only
    pub const LONG: Type = Type(3);
    pub const LONGLONG: Type = Type(8);
    pub const LONG_BLOB: Type = Type(251);
    pub const MEDIUM_BLOB: Type = Type(250);
    pub const NULL: Type = Type(6);
    pub const SET: Type = Type(248);
    pub const SHORT: Type = Type(2);
    pub const STRING: Type = Type(254);
    pub const TIME: Type = Type(11);
    pub const TIMESTAMP: Type = Type(7);
    pub const TINY: Type = Type(1);
    pub const TINY_BLOB: Type = Type(249);
    pub const VARCHAR: Type = Type(15);
    pub const VAR_STRING: Type = Type(253);
    pub const YEAR: Type = Type(13);
}

impl Default for Type {
    fn default() -> Type {
        Type::NULL
    }
}
