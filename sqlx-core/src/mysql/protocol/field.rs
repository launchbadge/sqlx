// https://mariadb.com/kb/en/library/resultset/#field-detail-flag
// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/group__group__cs__column__definition__flags.html
bitflags::bitflags! {
    pub struct FieldFlags: u16 {
        /// Field cannot be NULL
        const NOT_NULL = 1;

        /// Field is **part of** a primary key
        const PRIMARY_KEY = 2;

        /// Field is **part of** a unique key/constraint
        const UNIQUE_KEY = 4;

        /// Field is **part of** a unique or primary key
        const MULTIPLE_KEY = 8;

        /// Field is a blob.
        const BLOB = 16;

        /// Field is unsigned
        const UNISIGNED = 32;

        /// Field is zero filled.
        const ZEROFILL = 64;

        /// Field is binary (set for strings)
        const BINARY = 128;

        /// Field is an enumeration
        const ENUM = 256;

        /// Field is an auto-increment field
        const AUTO_INCREMENT = 512;

        /// Field is a timestamp
        const TIMESTAMP = 1024;

        /// Field is a set
        const SET = 2048;

        /// Field does not have a default value
        const NO_DEFAULT_VALUE = 4096;

        /// Field is set to NOW on UPDATE
        const ON_UPDATE_NOW = 8192;

        /// Field is a number
        const NUM = 32768;
    }
}
