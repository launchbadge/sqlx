// https://mariadb.com/kb/en/library/mariadb-connectorc-types-and-definitions/#server-status
bitflags::bitflags! {
    pub struct ServerStatusFlag: u16 {
        // A transaction is currently active
        const SERVER_STATUS_IN_TRANS = 1;

        // Autocommit mode is set
        const SERVER_STATUS_AUTOCOMMIT = 2;

        // more results exists (more packet follow)
        const SERVER_MORE_RESULTS_EXISTS = 8;

        const SERVER_QUERY_NO_GOOD_INDEX_USED = 16;
        const SERVER_QUERY_NO_INDEX_USED = 32;

        // when using COM_STMT_FETCH, indicate that current cursor still has result
        const SERVER_STATUS_CURSOR_EXISTS = 64;

        // when using COM_STMT_FETCH, indicate that current cursor has finished to send results
        const SERVER_STATUS_LAST_ROW_SENT = 128;

        // database has been dropped
        const SERVER_STATUS_DB_DROPPED = 1 << 8;

        // current escape mode is "no backslash escape"
        const SERVER_STATUS_NO_BACKSLASH_ESAPES = 1 << 9;

        // A DDL change did have an impact on an existing PREPARE (an
        // automatic reprepare has been executed)
        const SERVER_STATUS_METADATA_CHANGED = 1 << 10;

        // Last statement took more than the time value specified in
        // server variable long_query_time.
        const SERVER_QUERY_WAS_SLOW = 1 << 11;

        // this resultset contain stored procedure output parameter
        const SERVER_PS_OUT_PARAMS = 1 << 12;

        // current transaction is a read-only transaction
        const SERVER_STATUS_IN_TRANS_READONLY = 1 << 13;

        // session state change. see Session change type for more information
        const SERVER_SESSION_STATE_CHANGED = 1 << 14;
    }
}
