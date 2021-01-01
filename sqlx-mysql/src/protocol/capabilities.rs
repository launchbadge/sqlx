// https://dev.mysql.com/doc/internals/en/capability-flags.html#packet-Protocol::CapabilityFlags
// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/group__group__cs__capabilities__flags.html
// https://mariadb.com/kb/en/library/connection/#capabilities
bitflags::bitflags! {
    pub struct Capabilities: u64 {
        // use the improved version of "old password auth"
        // assumed to be set since 4.1
        const LONG_PASSWORD = 0x00000001;

        // send found (read: matched) rows instead of affected rows in the EOF packet
        const FOUND_ROWS = 0x00000002;

        // longer flags for column metadata
        // not used if PROTOCOL_41 is used (long flags are always received)
        const LONG_FLAG = 0x00000004;

        // database (schema) name can be specified on connect in Handshake Response Packet
        const CONNECT_WITH_DB = 0x00000008;

        // do not permit `database.table.column`
        const NO_SCHEMA = 0x00000010;

        // compression protocol supported
        // todo: expose in MySqlConnectOptions
        const COMPRESS = 0x00000020;

        // legacy flag to enable special ODBC handling
        // no handling since MySQL v3.22
        const ODBC = 0x00000040;

        // enable LOAD DATA LOCAL
        const LOCAL_FILES = 0x00000080;

        // SQL parser can ignore spaces before '('
        const IGNORE_SPACE = 0x00000100;

        // uses the 4.1+ protocol
        const PROTOCOL_41 = 0x00000200;

        // this is an interactive client
        // wait_timeout versus wait_interactive_timeout.
        const INTERACTIVE = 0x00000400;

        // use SSL encryption for this session
        const SSL = 0x00000800;

        // EOF packets will contain transaction status flags
        const TRANSACTIONS = 0x00002000;

        // support native 4.1+ authentication
        const SECURE_CONNECTION = 0x00008000;

        // can handle multiple statements in COM_QUERY and COM_STMT_PREPARE
        const MULTI_STATEMENTS = 0x00010000;

        // can send multiple result sets for COM_QUERY
        const MULTI_RESULTS = 0x00020000;

        // can send multiple result sets for COM_STMT_EXECUTE
        const PS_MULTI_RESULTS = 0x00040000;

        // supports authentication plugins
        const PLUGIN_AUTH = 0x00080000;

        // permits connection attributes
        const CONNECT_ATTRS = 0x00100000;

        // enable authentication response packet to be larger than 255 bytes.
        const PLUGIN_AUTH_LENENC_DATA = 0x00200000;

        // can handle connection for a user account with expired passwords
        const CAN_HANDLE_EXPIRED_PASSWORDS = 0x00400000;

        // capable of handling server state change information in an OK packet
        const SESSION_TRACK = 0x00800000;

        // client no longer needs EOF_Packet and will use OK_Packet instead.
        const DEPRECATE_EOF = 0x01000000;
    }
}
