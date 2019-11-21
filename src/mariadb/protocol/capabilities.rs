// https://mariadb.com/kb/en/library/connection/#capabilities
bitflags::bitflags! {
    pub struct Capabilities: u128 {
        const CLIENT_MYSQL = 1;
        const FOUND_ROWS = 2;

        // One can specify db on connect
        const CONNECT_WITH_DB = 8;

        // Can use compression protocol
        const COMPRESS = 32;

        // Can use LOAD DATA LOCAL
        const LOCAL_FILES = 128;

        // Ignore spaces before '('
        const IGNORE_SPACE = 256;

        // 4.1+ protocol
        const CLIENT_PROTOCOL_41 = 1 << 9;

        const CLIENT_INTERACTIVE = 1 << 10;

        // Can use SSL
        const SSL = 1 << 11;

        const TRANSACTIONS = 1 << 12;

        // 4.1+ authentication
        const SECURE_CONNECTION = 1 << 13;

        // Enable/disable multi-stmt support
        const MULTI_STATEMENTS = 1 << 16;

        // Enable/disable multi-results
        const MULTI_RESULTS = 1 << 17;

        // Enable/disable multi-results for PrepareStatement
        const PS_MULTI_RESULTS = 1 << 18;

        // Client supports plugin authentication
        const PLUGIN_AUTH = 1 << 19;

        // Client send connection attributes
        const CONNECT_ATTRS = 1 << 20;

        // Enable authentication response packet to be larger than 255 bytes
        const PLUGIN_AUTH_LENENC_CLIENT_DATA = 1 << 21;

        // Enable/disable session tracking in OK_Packet
        const CLIENT_SESSION_TRACK = 1 << 23;

        // EOF_Packet deprecation
        const CLIENT_DEPRECATE_EOF = 1 << 24;

        // Client support progress indicator (since 10.2)
        const MARIA_DB_CLIENT_PROGRESS = 1 << 32;

        // Permit COM_MULTI protocol
        const MARIA_DB_CLIENT_COM_MULTI = 1 << 33;

        // Permit bulk insert
        const MARIA_CLIENT_STMT_BULK_OPERATIONS = 1 << 34;
    }
}
