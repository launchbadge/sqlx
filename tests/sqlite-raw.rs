//! Tests for the raw (unprepared) query API for Sqlite.

use sqlx::{Cursor, Executor, Row, Sqlite};
use sqlx_test::new;
