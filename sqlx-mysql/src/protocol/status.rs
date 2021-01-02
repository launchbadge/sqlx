// https://dev.mysql.com/doc/internals/en/status-flags.html#packet-Protocol::StatusFlags
// https://dev.mysql.com/doc/dev/mysql-server/8.0.12/mysql__com_8h.html#a1d854e841086925be1883e4d7b4e8cad
// https://mariadb.com/kb/en/library/mariadb-connectorc-types-and-definitions/#server-status
bitflags::bitflags! {
    pub struct Status: u16 {
        // Is raised when a multi-statement transaction has been started, either explicitly,
        // by means of BEGIN or COMMIT AND CHAIN, or implicitly, by the first
        // transactional statement, when autocommit=off.
        const IN_TRANS = 0x0001;

        // Autocommit mode is set
        const AUTOCOMMIT = 0x0002;

        // Multi query - next query exists.
        const MORE_RESULTS_EXISTS = 0x0008;

        const NO_GOOD_INDEX_USED = 0x0010;
        const NO_INDEX_USED = 0x0020;

        // When using COM_STMT_FETCH, indicate that current cursor still has result
        const CURSOR_EXISTS = 0x0040;

        // When using COM_STMT_FETCH, indicate that current cursor has finished to send results
        const LAST_ROW_SENT = 0x0080;

        // Database has been dropped
        const DB_DROPPED = 0x0100;

        // Current escape mode is "no backslash escape"
        const NO_BACKSLASH_ESCAPES = 0x0200;

        // A DDL change did have an impact on an existing PREPARE (an automatic
        // re-prepare has been executed)
        const METADATA_CHANGED = 0x0400;

        // Last statement took more than the time value specified
        // in server variable long_query_time.
        const QUERY_WAS_SLOW = 0x0800;

        // This result-set contain stored procedure output parameter.
        const PS_OUT_PARAMS = 0x1000;

        // Current transaction is a read-only transaction.
        const IN_TRANS_READONLY = 0x2000;

        // This status flag, when on, implies that one of the state information has changed
        // on the server because of the execution of the last statement.
        const SESSION_STATE_CHANGED = 0x4000;
    }
}
