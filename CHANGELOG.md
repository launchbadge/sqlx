# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 - UNRELEASED

### Added

 - Results from the database are now zero-copy and no allocation beyond a shared read buffer
   for the TCP stream ( in other words, almost no per-query allocation ). Bind arguments still 
   do allocate a buffer per query.

 - [[#129]] Add support for [SQLite](https://sqlite.org/index.html). Generated code should be very close to normal use of the C API.

      * Adds `Sqlite`, `SqliteConnection`, `SqlitePool`, and other supporting types

 - [[#97]] [[#134]] Add support for user-defined types. [[@Freax13]]

      * Rust-only domain types or transparent wrappers around SQL types. These may be used _transparently_ inplace of
        the SQL type.

          ```rust
          #[derive(sqlx::Type)]
          #[repr(transparent)]
          struct Meters(i32);
          ```

      * Enumerations may be defined in Rust and can match SQL by integer discriminant or variant name.

          ```rust
          #[derive(sqlx::Type)]
          #[repr(i32)] // Expects a INT in SQL
          enum Color { Red = 1, Green = 2, Blue = 3 }
          ```

          ```rust
          #[derive(sqlx::Type)]
          #[sqlx(postgres(oid = 25))] // Postgres requires the OID
          #[sqlx(rename_all = "lowercase")] // similar to serde rename_all
          enum Color { Red, Green, Blue } // expects 'red', 'green', or 'blue'
          ```

      * **Postgres** further supports user-defined composite types.

          ```rust
          #[derive(sqlx::Type)]
          #[sqlx(postgres(oid = ?))] // Postgres requires the OID
          struct InterfaceType {
              name: String,
              supplier_id: i32,
              price: f64
          }
          ```

 - [[#98]] [[#131]] Add support for asynchronous notifications in Postgres (`LISTEN` / `NOTIFY`). [[@thedodd]]

      * Supports automatic reconnection on connection failure.

      * `PgListener` implements `Executor` and may be used to execute queries. Be careful however as if the
        intent is to handle and process messages rapidly you don't want to be tying up the connection
        for too long. Messages received during queries are buffered and will be delivered on the next call
        to `recv()`.

   ```rust
   let mut listener = PgListener::new(DATABASE_URL).await?;

   listener.listen("topic").await?;

   loop {
       let message = listener.recv().await?;

       println!("payload = {}", message.payload);
   }
   ```

### Changed

 - `Query` (and `QueryAs`; returned from `query()`, `query_as()`, `query!()`, and `query_as!()`) now will accept both `&mut Connection` or
   `&Pool` where as in 0.2.x they required `&mut &Pool`.

 - `Executor` now takes any value that implements `Execute` as a query. `Execute` is implemented for `Query` and `QueryAs` to mean
   exactly what they've meant so far, a prepared SQL query. However, `Execute` is also implemented for just `&str` which now performs
   a raw or unprepared SQL query. You can further use this to fetch `Row`s from the database though it is not as efficient as the
   prepared API (notably Postgres and MySQL send data back in TEXT mode as opposed to in BINARY mode).

   ```rust
   use sqlx::Executor;
   
   // Set the time zone parameter
   conn.execute("SET TIME ZONE LOCAL;").await

   // Demonstrate two queries at once with the raw API
   let mut cursor = conn.fetch("SELECT 1; SELECT 2");
   let row = cursor.next().await?.unwrap();
   let value: i32 = row.get(0); // 1
   let row = cursor.next().await?.unwrap();
   let value: i32 = row.get(0); // 2
   ```

 - `sqlx::Row` now has a lifetime (`'c`) tied to the database connection. In effect, this means that you cannot store `Row`s or collect
   them into a collection. `Query` (returned from `sqlx::query()`) has `map()` which takes a function to map from the `Row` to
   another type to make this transition easier.

   In 0.2.x

   ```rust
   let rows = sqlx::query("SELECT 1")
       .fetch_all(&mut conn).await?;
   ```

   In 0.3.x

   ```rust
   let values: Vec<i32> = sqlx::query("SELECT 1")
       .map(|row: PgRow| row.get(0))
       .fetch_all(&mut conn).await?;
   ```

   To assist with the above, `sqlx::query_as()` now supports querying directly into tuples (up to 9 elements).

   ```rust
   let values: Vec<(i32, bool)> = sqlx::query("SELECT 1, false")
       .fetch_all(&mut conn).await?;
   ```

 - `HasSqlType<T>: Database` is now `T: Type<Database>` to mirror `Encode` and `Decode`

 - `Query::fetch` (returned from `query()`) now returns a new `Cursor` type. `Cursor` is a custom `Stream` type where the
   item type borrows into the stream (which itself borrows from connection). This means that using `query().fetch()` you can now
   stream directly from the database with **zero-copy** and **zero-allocation**.

### Removed

 - `Query` (returned from `query()`) no longer has `fetch_one`, `fetch_optional`, or `fetch_all`. You _must_ map the row using `map()` and then
   you will have a `query::Map` value that has the former methods available.

   ```rust
   let values: Vec<i32> = sqlx::query("SELECT 1")
       .map(|row: PgRow| row.get(0))
       .fetch_all(&mut conn).await?;
   ```

### Fixed

 - [[#62]] [[#130]] [[#135]] Remove explicit set of `IntervalStyle`. Allow usage of SQLx for CockroachDB and potentially PgBouncer. [[@bmisiak]]

 - [[#108]] Allow nullable and borrowed values to be used as arguments in `query!` and `query_as!`. For example, where the column would
   resolve to `String` in Rust (TEXT, VARCHAR, etc.), you may now use `Option<String>`, `Option<&str>`, or `&str` instead. [[@abonander]]

 - [[#108]] Make unknown type errors far more informative. As an example, trying to `SELECT` a `DATE` column will now try and tell you about the
   `chrono` feature. [[@abonander]]

   ```
   optional feature `chrono` required for type DATE of column #1 ("now")
   ```

[#62]: https://github.com/launchbadge/sqlx/issues/62
[#130]: https://github.com/launchbadge/sqlx/issues/130

[#98]: https://github.com/launchbadge/sqlx/pull/98
[#97]: https://github.com/launchbadge/sqlx/pull/97
[#134]: https://github.com/launchbadge/sqlx/pull/134
[#129]: https://github.com/launchbadge/sqlx/pull/129
[#131]: https://github.com/launchbadge/sqlx/pull/131
[#135]: https://github.com/launchbadge/sqlx/pull/135
[#108]: https://github.com/launchbadge/sqlx/pull/108

[@bmisiak]: https://github.com/bmisiak

## 0.2.6 - 2020-03-10

### Added

 - [[#114]] Export `sqlx_core::Transaction` [[@thedodd]]

### Fixed

 - [[#125]] [[#126]] Fix statement execution in MySQL if it contains NULL statement values [[@repnop]]

 - [[#105]] [[#109]] Allow trailing commas in query macros [[@timmythetiny]]

[#105]: https://github.com/launchbadge/sqlx/pull/105
[#109]: https://github.com/launchbadge/sqlx/pull/109
[#114]: https://github.com/launchbadge/sqlx/pull/114
[#125]: https://github.com/launchbadge/sqlx/pull/125
[#126]: https://github.com/launchbadge/sqlx/pull/126

[@timmythetiny]: https://github.com/timmythetiny
[@thedodd]: https://github.com/thedodd

## 0.2.5 - 2020-02-01

### Fixed

 - Fix decoding of Rows containing NULLs in Postgres [#104]

 - After a large review and some battle testing by [@ianthetechie](https://github.com/ianthetechie)
   of the `Pool`, a live leaking issue was found. This has now been fixed by [@abonander] in [#84] which
   included refactoring to make the pool internals less brittle (using RAII instead of manual
   work is one example) and to help any future contributors when changing the pool internals.

 - Passwords are now being precent decoding before being presented to the server [[@repnop]]

 - [@100] Fix `FLOAT` and `DOUBLE` decoding in MySQL

[#84]: https://github.com/launchbadge/sqlx/issues/84
[#100]: https://github.com/launchbadge/sqlx/issues/100
[#104]: https://github.com/launchbadge/sqlx/issues/104

[@repnop]: https://github.com/repnop

### Added

 - [[#72]] Add `PgTypeInfo::with_oid` to allow simple construction of `PgTypeInfo` which enables `HasSqlType`
   to be implemented by downstream consumers of SQLx [[@jplatte]]

 - [[#96]] Add support for returning columns from `query!` with a name of a rust keyword by
   using raw identifiers [[@yaahc]]

 - [[#71]] Implement derives for `Encode` and `Decode`. This is the first step to supporting custom types in SQLx. [[@Freax13]]

[#72]: https://github.com/launchbadge/sqlx/issues/72
[#96]: https://github.com/launchbadge/sqlx/issues/96
[#71]: https://github.com/launchbadge/sqlx/issues/71

[@jplatte]: https://github.com/jplatte
[@yaahc]: https://github.com/yaahc
[@Freax13]: https://github.com/Freax13

## 0.2.4 - 2020-01-18

### Fixed

 - Fix decoding of Rows containing NULLs in MySQL (and add an integration test so this doesn't break again)

## 0.2.3 - 2020-01-18

### Fixed

 - Fix `query!` when used on a query that does not return results

## 0.2.2 - 2020-01-16

### Added

 - [[#57]] Add support for unsigned integers and binary types in `query!` for MySQL [[@mehcode]]

[#57]: https://github.com/launchbadge/sqlx/issues/57

### Fixed

 - Fix stall when requesting TLS from a Postgres server that explicitly does not support TLS (such as postgres running inside docker) [[@abonander]]

 - [[#66]] Declare used features for `tokio` in `sqlx-macros` explicitly

[#66]: https://github.com/launchbadge/sqlx/issues/66

## 0.2.1 - 2020-01-16

### Fixed

 - [[#64], [#65]] Fix decoding of Rows containing NULLs in MySQL [[@danielakhterov]]

[#64]: https://github.com/launchbadge/sqlx/pull/64
[#65]: https://github.com/launchbadge/sqlx/pull/65

 - [[#55]] Use a shared tokio runtime for the `query!` macro compile-time execution (under the `runtime-tokio` feature) [[@udoprog]]

[#55]: https://github.com/launchbadge/sqlx/pull/55

## 0.2.0 - 2020-01-15

### Fixed

 - https://github.com/launchbadge/sqlx/issues/47

### Added

 - Support Tokio through an optional `runtime-tokio` feature.

 - Support SQL transactions. You may now use the `begin()` function on `Pool` or `Connection` to
   start a new SQL transaction. This returns `sqlx::Transaction` which will `ROLLBACK` on `Drop`
   or can be explicitly `COMMIT` using `commit()`.

 - Support TLS connections.

## 0.1.4 - 2020-01-11

### Fixed

 - https://github.com/launchbadge/sqlx/issues/43

 - https://github.com/launchbadge/sqlx/issues/40

### Added

 - Support for `SCRAM-SHA-256` authentication in Postgres [#37](https://github.com/launchbadge/sqlx/pull/37) [@danielakhterov](https://github.com/danielakhterov)

 - Implement `Debug` for Pool [#42](https://github.com/launchbadge/sqlx/pull/42) [@prettynatty](https://github.com/prettynatty)

## 0.1.3 - 2020-01-06

### Fixed

 - https://github.com/launchbadge/sqlx/issues/30

## 0.1.2 - 2020-01-03

### Added

 - Support for Authentication in MySQL 5+ including the newer authentication schemes now default in MySQL 8: `mysql_native_password`, `sha256_password`, and `caching_sha2_password`.

 - [`Chrono`](https://github.com/chronotope/chrono) support for MySQL was only partially implemented (was missing `NaiveTime` and `DateTime<Utc>`).

 - `Vec<u8>` (and `[u8]`) support for MySQL (`BLOB`) and Postgres (`BYTEA`).

[@abonander]: https://github.com/abonander
[@danielakhterov]: https://github.com/danielakhterov
[@mehcode]: https://github.com/mehcode
[@udoprog]: https://github.com/udoprog
