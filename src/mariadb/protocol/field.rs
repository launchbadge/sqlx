// https://mariadb.com/kb/en/library/resultset/#field-types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldType(pub u8);

impl FieldType {
    pub const MYSQL_TYPE_BIT: FieldType = FieldType(16);
    pub const MYSQL_TYPE_BLOB: FieldType = FieldType(252);
    pub const MYSQL_TYPE_DATE: FieldType = FieldType(10);
    pub const MYSQL_TYPE_DATETIME: FieldType = FieldType(12);
    pub const MYSQL_TYPE_DATETIME2: FieldType = FieldType(18);
    pub const MYSQL_TYPE_DECIMAL: FieldType = FieldType(0);
    pub const MYSQL_TYPE_DOUBLE: FieldType = FieldType(5);
    pub const MYSQL_TYPE_ENUM: FieldType = FieldType(247);
    pub const MYSQL_TYPE_FLOAT: FieldType = FieldType(4);
    pub const MYSQL_TYPE_GEOMETRY: FieldType = FieldType(255);
    pub const MYSQL_TYPE_INT24: FieldType = FieldType(9);
    pub const MYSQL_TYPE_JSON: FieldType = FieldType(245);
    pub const MYSQL_TYPE_LONG: FieldType = FieldType(3);
    pub const MYSQL_TYPE_LONGLONG: FieldType = FieldType(8);
    pub const MYSQL_TYPE_LONG_BLOB: FieldType = FieldType(251);
    pub const MYSQL_TYPE_MEDIUM_BLOB: FieldType = FieldType(250);
    pub const MYSQL_TYPE_NEWDATE: FieldType = FieldType(14);
    pub const MYSQL_TYPE_NEWDECIMAL: FieldType = FieldType(246);
    pub const MYSQL_TYPE_NULL: FieldType = FieldType(6);
    pub const MYSQL_TYPE_SET: FieldType = FieldType(248);
    pub const MYSQL_TYPE_SHORT: FieldType = FieldType(2);
    pub const MYSQL_TYPE_STRING: FieldType = FieldType(254);
    pub const MYSQL_TYPE_TIME: FieldType = FieldType(11);
    pub const MYSQL_TYPE_TIME2: FieldType = FieldType(19);
    pub const MYSQL_TYPE_TIMESTAMP: FieldType = FieldType(7);
    pub const MYSQL_TYPE_TIMESTAMP2: FieldType = FieldType(17);
    pub const MYSQL_TYPE_TINY: FieldType = FieldType(1);
    pub const MYSQL_TYPE_TINY_BLOB: FieldType = FieldType(249);
    pub const MYSQL_TYPE_VARCHAR: FieldType = FieldType(15);
    pub const MYSQL_TYPE_VAR_STRING: FieldType = FieldType(253);
    pub const MYSQL_TYPE_YEAR: FieldType = FieldType(13);
}

// https://mariadb.com/kb/en/library/com_stmt_execute/#parameter-flag
bitflags::bitflags! {
    pub struct ParameterFlag: u8 {
        const UNSIGNED = 128;
    }
}

// https://mariadb.com/kb/en/library/resultset/#field-detail-flag
bitflags::bitflags! {
    pub struct FieldDetailFlag: u16 {
        const NOT_NULL = 1;
        const PRIMARY_KEY = 2;
        const UNIQUE_KEY = 4;
        const MULTIPLE_KEY = 8;
        const BLOB = 16;
        const UNSIGNED = 32;
        const ZEROFILL_FLAG = 64;
        const BINARY_COLLATION = 128;
        const ENUM = 256;
        const AUTO_INCREMENT = 512;
        const TIMESTAMP = 1024;
        const SET = 2048;
        const NO_DEFAULT_VALUE_FLAG = 4096;
        const ON_UPDATE_NOW_FLAG = 8192;
        const NUM_FLAG = 32768;
    }
}
