bitflags::bitflags! {
    /// <https://mariadb.com/kb/en/result-set-packets/#column-details-flag>
    /// <https://github.com/mysql/mysql-server/blob/7ed30a748964c009d4909cb8b4b22036ebdef239/router/src/mysql_protocol/include/mysqlrouter/classic_protocol_constants.h#L260-L274>
    pub(crate) struct ColumnFlags: u16 {
        /// Field can't be `NULL`.
        const NOT_NULL = 1;

        /// Field is part of a primary key.
        const PRIMARY_KEY = 2;

        /// Field is part of a unique key.
        const UNIQUE_KEY = 4;

        /// Field is part of a multi-part unique or primary key.
        const MULTIPLE_KEY = 8;

        /// Field is a blob.
        const BLOB = 16;

        /// Field is unsigned.
        const UNSIGNED = 32;

        /// Field is zero filled.
        const ZEROFILL = 64;

        /// Field has a binary collation.
        const BINARY_COLLATION = 128;

        /// Field is an enumeration.
        const ENUM = 256;

        /// Field is an auto-increment field.
        const AUTO_INCREMENT = 512;

        /// Field is a timestamp.
        const TIMESTAMP = 1024;

        /// Field is a set.
        const SET = 2048;

        /// Field does not have a default value.
        const NO_DEFAULT_VALUE = 4096;

        /// Field is set to NOW on UPDATE.
        const ON_UPDATE_NOW = 8192;

        /// Field is a number.
        const NUM = 32768;
    }
}
