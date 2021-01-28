use std::fmt::{self, Debug, Formatter};

use crate::protocol::OkPacket;

/// Represents the execution result of an operation on the database server.
///
/// Returned from [`execute()`][sqlx_core::Executor::execute].
///
#[allow(clippy::module_name_repetitions)]
pub struct MySqlQueryResult(OkPacket);

impl MySqlQueryResult {
    /// Returns the number of rows changed, deleted, or inserted by the statement
    /// if it was an `UPDATE`, `DELETE` or `INSERT`. For `SELECT` statements, returns
    /// the number of rows returned.
    ///
    /// For more information, see the corresponding method in the official C API:
    /// <https://dev.mysql.com/doc/c-api/8.0/en/mysql-affected-rows.html>
    ///
    #[doc(alias = "affected_rows")]
    #[must_use]
    pub const fn rows_affected(&self) -> u64 {
        self.0.affected_rows
    }

    /// Return the number of rows matched by the `UPDATE` statement.
    ///
    /// This is in contrast to [`rows_affected()`] which will return the number
    /// of rows actually changed by the `UPDATE statement.
    ///
    /// Returns `0` for all other statements.
    ///
    #[must_use]
    pub const fn rows_matched(&self) -> u64 {
        self.0.info.matched
    }

    /// Returns the number of rows processed by the multi-row `INSERT`
    /// or `ALTER TABLE` statement.
    ///
    /// For multi-row `INSERT`, this is not necessarily the number of rows actually
    /// inserted because [`duplicates()`] can be non-zero.
    ///
    /// For `ALTER TABLE`, this is the number of rows that were copied while
    /// making alterations.
    ///
    /// Returns `0` for all other statements.
    ///
    #[must_use]
    pub const fn records(&self) -> u64 {
        self.0.info.records
    }

    /// Returns the number of rows that could not be inserted by a multi-row `INSERT`
    /// statement because they would duplicate some existing unique index value.
    ///
    /// Returns `0` for all other statements.
    ///
    #[must_use]
    pub const fn duplicates(&self) -> u64 {
        self.0.info.duplicates
    }

    /// Returns the integer generated for an `AUTO_INCREMENT` column by the
    /// `INSERT` statement.
    ///
    /// When inserting multiple rows, returns the id of the _first_ row in
    /// set of inserted rows.
    ///
    /// For more information, see the corresponding method in the official C API:
    /// <https://dev.mysql.com/doc/c-api/8.0/en/mysql-insert-id.html>
    ///
    #[doc(alias = "last_insert_id")]
    #[must_use]
    pub const fn inserted_id(&self) -> Option<u64> {
        // NOTE: a valid ID is never zero
        if self.0.last_insert_id == 0 { None } else { Some(self.0.last_insert_id) }
    }

    /// Returns the number of errors, warnings, and notes generated during
    /// execution of the statement.
    ///
    /// To read the warning messages, execute
    /// the [`SHOW WARNINGS`](https://dev.mysql.com/doc/refman/8.0/en/show-warnings.html)
    /// statement on the same connection (and before executing any other statements).
    ///
    /// As an example, the statement `SELECT 1/0` will execute successfully and return `NULL` but
    /// indicate 1 warning.
    ///
    #[doc(alias = "warnings_count")]
    #[must_use]
    pub const fn warnings(&self) -> u16 {
        self.0.warnings
    }
}

impl Debug for MySqlQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlQueryResult")
            .field("inserted_id", &self.inserted_id())
            .field("rows_affected", &self.rows_affected())
            .field("rows_matched", &self.rows_matched())
            .field("records", &self.records())
            .field("duplicates", &self.duplicates())
            .field("warnings", &self.warnings())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use conquer_once::Lazy;
    use sqlx_core::io::Deserialize;

    use super::MySqlQueryResult;
    use crate::protocol::{Capabilities, OkPacket};

    static CAPABILITIES: Lazy<Capabilities> = Lazy::new(|| {
        Capabilities::PROTOCOL_41 | Capabilities::SESSION_TRACK | Capabilities::TRANSACTIONS
    });

    #[test]
    fn insert_1() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x01\x01\x02\0\0\0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(res.inserted_id(), Some(1));

        Ok(())
    }

    #[test]
    fn insert_5() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x05\x02\x02\0\0\0&Records: 5  Duplicates: 0  Warnings: 0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 5);
        assert_eq!(res.inserted_id(), Some(2));
        assert_eq!(res.records(), 5);
        assert_eq!(res.duplicates(), 0);

        Ok(())
    }

    #[test]
    fn insert_5_or_update_3() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x08\x07\x02\0\0\0&Records: 5  Duplicates: 3  Warnings: 0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 8);
        assert_eq!(res.inserted_id(), Some(7));
        assert_eq!(res.records(), 5);
        assert_eq!(res.duplicates(), 3);

        Ok(())
    }

    #[test]
    fn update_7_change_3() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x03\0\"\0\0\0(Rows matched: 7  Changed: 3  Warnings: 0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 3);
        assert_eq!(res.inserted_id(), None);
        assert_eq!(res.rows_matched(), 7);

        Ok(())
    }

    #[test]
    fn update_1_change_1() -> anyhow::Result<()> {
        let packet =
            Bytes::from(&b"\0\x01\0\x02\0\0\0(Rows matched: 1  Changed: 1  Warnings: 0"[..]);

        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(res.inserted_id(), None);
        assert_eq!(res.rows_matched(), 1);

        Ok(())
    }

    #[test]
    fn delete_1() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x01\0\x02\0\0\0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 1);
        assert_eq!(res.inserted_id(), None);

        Ok(())
    }

    #[test]
    fn delete_6() -> anyhow::Result<()> {
        let packet = Bytes::from(&b"\0\x06\0\"\0\0\0"[..]);
        let ok = OkPacket::deserialize_with(packet, *CAPABILITIES)?;
        let res = MySqlQueryResult(ok);

        assert_eq!(res.rows_affected(), 6);
        assert_eq!(res.inserted_id(), None);

        Ok(())
    }
}
