# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0 - 2020-11-12

_to be written_

## 0.4.0-beta.1 - 2020-07-27

### Highlights

-   Enable compile-time type checking from cached metadata to enable building
    in an environment without access to a development database (e.g., Docker, CI).

-   Initial support for **Microsoft SQL Server**. If there is something missing that you need,
    open an issue. We are happy to help.

-   SQL migrations, both with a CLI tool and programmatically loading migrations at runtime.

-   Runtime-determined database driver, `Any`, to support compile-once and run with a database
    driver selected at runtime.

-   Support for user-defined types and more generally overriding the inferred Rust type from SQL
    with compile-time SQL verification.

### Fixed

#### MySQL

-   [[#418]] Support zero dates and times [[@blackwolf12333]]

### Added

-   [[#174]] Inroduce a builder to construct connections to bypass the URI parsing

    ```rust
    // MSSQL
    let conn = MssqlConnectOptions::new()
        .host("localhost")
        .database("master")
        .username("sa")
        .password("Password")
        .connect().await?;

    // SQLite
    let conn = SqliteConnectOptions::from_str("sqlite://a.db")?
        .foreign_keys(false)
        .connect().await?;
    ```

-   [[#127]] Get the last ID or Row ID inserted for MySQL or SQLite

    ```rust
    // MySQL
    let id: u64 = query!("INSERT INTO table ( col ) VALUES ( ? )", val)
        .execute(&mut conn).await?
        .last_insert_id(); // LAST_INSERT_ID()

    // SQLite
    let id: i64 = query!("INSERT INTO table ( col ) VALUES ( ?1 )", val)
        .execute(&mut conn).await?
        .last_insert_rowid(); // sqlite3_last_insert_rowid()
    ```

-   [[#263]] Add hooks to the Pool: `after_connect`, `before_release`, and `after_acquire`

    ```rust
    // PostgreSQL
    let pool = PgPoolOptions::new()
        .after_connect(|conn| Box::pin(async move {
            conn.execute("SET application_name = 'your_app';").await?;
            conn.execute("SET search_path = 'my_schema';").await?;

            Ok(())
        }))
        .connect("postgres:// …").await?
    ```

-   [[#308]] [[#495]] Extend `derive(FromRow)` with support for `#[sqlx(default)]` on fields to allow reading in a partial query [[@OriolMunoz]]

-   [[#454]] [[#456]] Support `rust_decimal::Decimal` as an alternative to `bigdecimal::BigDecimal` for `NUMERIC` columns in MySQL and PostgreSQL [[@pimeys]]

-   [[#181]] Column names and type information is now accessible from `Row` via `Row::columns()` or `Row::column(name)`

#### PostgreSQL

-   [[#197]] [[#271]] Add initial support for `INTERVAL` (full support pending a `time::Period` type) [[@dimtion]]

#### MySQL

-   [[#449]] [[#450]] Support Unix Domain Sockets (UDS) for MySQL [[@pimeys]]

#### SQLite

-   Types are now inferred for expressions. This means its now possible to use `query!` and `query_as!` for:

    ```rust
    let row = query!("SELECT 10 as _1, x + 5 as _2 FROM table").fetch_one(&mut conn).await?;

    assert_eq!(row._1, 10);
    assert_eq!(row._2, 5); // 5 + x?
    ```

-   [[#167]] Support `foreign_keys` explicitly with a `foreign_keys(true)` method available on `SqliteConnectOptions` which is a builder
    for new SQLite connections (and can be passed into `PoolOptions` to build a pool).

    ```rust
    let conn = SqliteConnectOptions::new()
        .foreign_keys(true) // on by default
        .connect().await?;
    ```

-   [[#430]] [[#438]] Add method to get the raw SQLite connection handle [[@agentsim]]

    ```rust
    // conn is `SqliteConnection`
    // this is not unsafe, but what you do with the handle will be
    let ptr: *mut libsqlite3::sqlite3 = conn.as_raw_handle();
    ```

-   [[#164]] Support `TIMESTAMP`, `DATETIME`, `DATE`, and `TIME` via `chrono` in SQLite [[@felipesere]] [[@meteficha]]

### Changed

-   `Transaction` now mutably borrows a connection instead of owning it. This enables a new (or nested) transaction to be started from `&mut conn`.

-   [[#145]] [[#444]] Use a least-recently-used (LRU) cache to limit the growth of the prepared statement cache for SQLite, MySQL, and PostgreSQL [[@pimeys]]

#### SQLite

-   [[#499]] `INTEGER` now resolves to `i64` instead of `i32`, `INT4` will still resolve to `i32`

### Removed

[#127]: https://github.com/launchbadge/sqlx/issues/127
[#174]: https://github.com/launchbadge/sqlx/issues/174
[#145]: https://github.com/launchbadge/sqlx/issues/145
[#164]: https://github.com/launchbadge/sqlx/issues/164
[#167]: https://github.com/launchbadge/sqlx/issues/167
[#181]: https://github.com/launchbadge/sqlx/issues/181
[#197]: https://github.com/launchbadge/sqlx/issues/197
[#263]: https://github.com/launchbadge/sqlx/issues/263
[#308]: https://github.com/launchbadge/sqlx/issues/308
[#418]: https://github.com/launchbadge/sqlx/issues/418
[#430]: https://github.com/launchbadge/sqlx/issues/430
[#449]: https://github.com/launchbadge/sqlx/issues/449
[#499]: https://github.com/launchbadge/sqlx/issues/499
[#454]: https://github.com/launchbadge/sqlx/issues/454
[#271]: https://github.com/launchbadge/sqlx/pull/271
[#444]: https://github.com/launchbadge/sqlx/pull/444
[#438]: https://github.com/launchbadge/sqlx/pull/438
[#495]: https://github.com/launchbadge/sqlx/pull/495
[#495]: https://github.com/launchbadge/sqlx/pull/495

## 0.3.5 - 2020-05-06

### Fixed

-   [[#259]] Handle percent-encoded paths for SQLite [[@g-s-k]]

-   [[#281]] Deallocate SQLite statements before closing the SQLite connection [[@hasali19]]

-   [[#284]] Fix handling of `0` for `BigDecimal` in PostgreSQL and MySQL [[@abonander]]

### Added

-   [[#256]] Add `query_unchecked!` and `query_file_unchecked!` with similar semantics to `query_as_unchecked!` [[@meh]]

-   [[#252]] [[#297]] Derive serveral traits for the `Json<T>` wrapper type [[@meh]]

-   [[#261]] Add support for `#[sqlx(rename_all = "snake_case")]` to `#[derive(Type)]` [[@shssoichiro]]

-   [[#253]] Add support for UNIX domain sockets to PostgreSQL [[@Nilix007]]

-   [[#251]] Add support for textual JSON on MySQL [[@blackwolf12333]]

-   [[#275]] [[#268]] Optionally log formatted SQL queries on execution [[@shssoichiro]]

-   [[#267]] Support Cargo.toml relative `.env` files; allows for each crate in a workspace to use their own `.env` file and thus their own `DATABASE_URL` [[@xyzd]]

[#252]: https://github.com/launchbadge/sqlx/pull/252
[#261]: https://github.com/launchbadge/sqlx/pull/261
[#256]: https://github.com/launchbadge/sqlx/pull/256
[#259]: https://github.com/launchbadge/sqlx/pull/259
[#253]: https://github.com/launchbadge/sqlx/pull/253
[#297]: https://github.com/launchbadge/sqlx/pull/297
[#251]: https://github.com/launchbadge/sqlx/pull/251
[#275]: https://github.com/launchbadge/sqlx/pull/275
[#267]: https://github.com/launchbadge/sqlx/pull/267
[#268]: https://github.com/launchbadge/sqlx/pull/268
[#281]: https://github.com/launchbadge/sqlx/pull/281
[#284]: https://github.com/launchbadge/sqlx/pull/284

## 0.3.4 - 2020-04-10

### Fixed

-   [[#241]] Type name for custom enum is not always attached to TypeInfo in PostgreSQL

-   [[#237]] [[#238]] User-defined type name matching is now case-insensitive in PostgreSQL [[@qtbeee]]

-   [[#231]] Handle empty queries (and those with comments) in SQLite

-   [[#228]] Provide `MapRow` implementations for functions (enables `.map(|row| ...)` over `.try_map(|row| ...)`)

### Added

-   [[#234]] Add support for `NUMERIC` in MySQL with the `bigdecimal` crate [[@xiaopengli89]]

-   [[#227]] Support `#[sqlx(rename = "new_name")]` on struct fields within a `FromRow` derive [[@sidred]]

[#228]: https://github.com/launchbadge/sqlx/issues/228
[#231]: https://github.com/launchbadge/sqlx/issues/231
[#237]: https://github.com/launchbadge/sqlx/issues/237
[#241]: https://github.com/launchbadge/sqlx/issues/241
[#227]: https://github.com/launchbadge/sqlx/pull/227
[#234]: https://github.com/launchbadge/sqlx/pull/234
[#238]: https://github.com/launchbadge/sqlx/pull/238

## 0.3.3 - 2020-04-01

### Fixed

-   [[#214]] Handle percent-encoded usernames in a database URL [[@jamwaffles]]

### Changed

-   [[#216]] Mark `Cursor`, `Query`, `QueryAs`, `query::Map`, and `Transaction` as `#[must_use]` [[@Ace4896]]

-   [[#213]] Remove matches dependency and use matches macro from std [[@nrjais]]

[#216]: https://github.com/launchbadge/sqlx/pull/216
[#214]: https://github.com/launchbadge/sqlx/pull/214
[#213]: https://github.com/launchbadge/sqlx/pull/213

## 0.3.2 - 2020-03-31

### Fixed

-   [[#212]] Removed sneaky `println!` in `MySqlCursor`

[#212]: https://github.com/launchbadge/sqlx/issues/212

## 0.3.1 - 2020-03-30

### Fixed

-   [[#203]] Allow an empty password for MySQL

-   [[#204]] Regression in error reporting for invalid SQL statements on PostgreSQL

-   [[#200]] Fixes the incorrect handling of raw (`r#...`) fields of a struct in the `FromRow` derive [[@sidred]]

[#200]: https://github.com/launchbadge/sqlx/pull/200
[#203]: https://github.com/launchbadge/sqlx/issues/203
[#204]: https://github.com/launchbadge/sqlx/issues/204

## 0.3.0 - 2020-03-29

### Breaking Changes

-   `sqlx::Row` now has a lifetime (`'c`) tied to the database connection. In effect, this means that you cannot store `Row`s or collect
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

    To assist with the above, `sqlx::query_as()` now supports querying directly into tuples (up to 9 elements) or
    struct types with a `#[derive(FromRow)]`.

    ```rust
    // This extension trait is needed until a rust bug is fixed
    use sqlx::postgres::PgQueryAs;

    let values: Vec<(i32, bool)> = sqlx::query_as("SELECT 1, false")
        .fetch_all(&mut conn).await?;
    ```

-   `HasSqlType<T>: Database` is now `T: Type<Database>` to mirror `Encode` and `Decode`

-   `Query::fetch` (returned from `query()`) now returns a new `Cursor` type. `Cursor` is a Stream-like type where the
    item type borrows into the stream (which itself borrows from connection). This means that using `query().fetch()` you can now
    stream directly from the database with **zero-copy** and **zero-allocation**.

-   Remove `PgTypeInfo::with_oid` and replace with `PgTypeInfo::with_name`

### Added

-   Results from the database are now zero-copy and no allocation beyond a shared read buffer
    for the TCP stream ( in other words, almost no per-query allocation ). Bind arguments still
    do allocate a buffer per query.

-   [[#129]] Add support for [SQLite](https://sqlite.org/index.html). Generated code should be very close to normal use of the C API.

    -   Adds `Sqlite`, `SqliteConnection`, `SqlitePool`, and other supporting types

-   [[#97]] [[#134]] Add support for user-defined types. [[@Freax13]]

    -   Rust-only domain types or transparent wrappers around SQL types. These may be used _transparently_ inplace of
        the SQL type.

        ```rust
        #[derive(sqlx::Type)]
        #[repr(transparent)]
        struct Meters(i32);
        ```

    -   Enumerations may be defined in Rust and can match SQL by integer discriminant or variant name.

        ```rust
        #[derive(sqlx::Type)]
        #[repr(i32)] // Expects a INT in SQL
        enum Color { Red = 1, Green = 2, Blue = 3 }
        ```

        ```rust
        #[derive(sqlx::Type)]
        #[sqlx(rename = "TEXT")] // May also be the name of a user defined enum type
        #[sqlx(rename_all = "lowercase")] // similar to serde rename_all
        enum Color { Red, Green, Blue } // expects 'red', 'green', or 'blue'
        ```

    -   **Postgres** further supports user-defined composite types.

        ```rust
        #[derive(sqlx::Type)]
        #[sqlx(rename = "interface_type")]
        struct InterfaceType {
            name: String,
            supplier_id: i32,
            price: f64
        }
        ```

-   [[#98]] [[#131]] Add support for asynchronous notifications in Postgres (`LISTEN` / `NOTIFY`). [[@thedodd]]

    -   Supports automatic reconnection on connection failure.

    -   `PgListener` implements `Executor` and may be used to execute queries. Be careful however as if the
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

-   Add _unchecked_ variants of the query macros. These will still verify the SQL for syntactic and
    semantic correctness with the current database but they will not check the input or output types.

    This is intended as a temporary solution until `query_as!` is able to support user defined types.

    -   `query_as_unchecked!`
    -   `query_file_as_unchecked!`

-   Add support for many more types in Postgres

    -   `JSON`, `JSONB` [[@oeb25]]
    -   `INET`, `CIDR` [[@PoiScript]]
    -   Arrays [[@oeb25]]
    -   Composites ( Rust tuples or structs with a `#[derive(Type)]` )
    -   `NUMERIC` [[@abonander]]
    -   `OID` (`u32`)
    -   `"CHAR"` (`i8`)
    -   `TIMESTAMP`, `TIMESTAMPTZ`, etc. with the `time` crate [[@utter-step]]
    -   Enumerations ( Rust enums with a `#[derive(Type)]` ) [[@Freax13]]

### Changed

-   `Query` (and `QueryAs`; returned from `query()`, `query_as()`, `query!()`, and `query_as!()`) now will accept both `&mut Connection` or
    `&Pool` where as in 0.2.x they required `&mut &Pool`.

-   `Executor` now takes any value that implements `Execute` as a query. `Execute` is implemented for `Query` and `QueryAs` to mean
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

### Removed

-   `Query` (returned from `query()`) no longer has `fetch_one`, `fetch_optional`, or `fetch_all`. You _must_ map the row using `map()` and then
    you will have a `query::Map` value that has the former methods available.

    ```rust
    let values: Vec<i32> = sqlx::query("SELECT 1")
        .map(|row: PgRow| row.get(0))
        .fetch_all(&mut conn).await?;
    ```

### Fixed

-   [[#62]] [[#130]] [[#135]] Remove explicit set of `IntervalStyle`. Allow usage of SQLx for CockroachDB and potentially PgBouncer. [[@bmisiak]]

-   [[#108]] Allow nullable and borrowed values to be used as arguments in `query!` and `query_as!`. For example, where the column would
    resolve to `String` in Rust (TEXT, VARCHAR, etc.), you may now use `Option<String>`, `Option<&str>`, or `&str` instead. [[@abonander]]

-   [[#108]] Make unknown type errors far more informative. As an example, trying to `SELECT` a `DATE` column will now try and tell you about the
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

## 0.2.6 - 2020-03-10

### Added

-   [[#114]] Export `sqlx_core::Transaction` [[@thedodd]]

### Fixed

-   [[#125]] [[#126]] Fix statement execution in MySQL if it contains NULL statement values [[@repnop]]

-   [[#105]] [[#109]] Allow trailing commas in query macros [[@timmythetiny]]

[#105]: https://github.com/launchbadge/sqlx/pull/105
[#109]: https://github.com/launchbadge/sqlx/pull/109
[#114]: https://github.com/launchbadge/sqlx/pull/114
[#125]: https://github.com/launchbadge/sqlx/pull/125
[#126]: https://github.com/launchbadge/sqlx/pull/126
[@timmythetiny]: https://github.com/timmythetiny
[@thedodd]: https://github.com/thedodd

## 0.2.5 - 2020-02-01

### Fixed

-   Fix decoding of Rows containing NULLs in Postgres [#104]

-   After a large review and some battle testing by [@ianthetechie](https://github.com/ianthetechie)
    of the `Pool`, a live leaking issue was found. This has now been fixed by [@abonander] in [#84] which
    included refactoring to make the pool internals less brittle (using RAII instead of manual
    work is one example) and to help any future contributors when changing the pool internals.

-   Passwords are now being precent decoding before being presented to the server [[@repnop]]

-   [@100] Fix `FLOAT` and `DOUBLE` decoding in MySQL

[#84]: https://github.com/launchbadge/sqlx/issues/84
[#100]: https://github.com/launchbadge/sqlx/issues/100
[#104]: https://github.com/launchbadge/sqlx/issues/104

### Added

-   [[#72]] Add `PgTypeInfo::with_oid` to allow simple construction of `PgTypeInfo` which enables `HasSqlType`
    to be implemented by downstream consumers of SQLx [[@jplatte]]

-   [[#96]] Add support for returning columns from `query!` with a name of a rust keyword by
    using raw identifiers [[@yaahc]]

-   [[#71]] Implement derives for `Encode` and `Decode`. This is the first step to supporting custom types in SQLx. [[@Freax13]]

[#72]: https://github.com/launchbadge/sqlx/issues/72
[#96]: https://github.com/launchbadge/sqlx/issues/96
[#71]: https://github.com/launchbadge/sqlx/issues/71

## 0.2.4 - 2020-01-18

### Fixed

-   Fix decoding of Rows containing NULLs in MySQL (and add an integration test so this doesn't break again)

## 0.2.3 - 2020-01-18

### Fixed

-   Fix `query!` when used on a query that does not return results

## 0.2.2 - 2020-01-16

### Added

-   [[#57]] Add support for unsigned integers and binary types in `query!` for MySQL [[@mehcode]]

[#57]: https://github.com/launchbadge/sqlx/issues/57

### Fixed

-   Fix stall when requesting TLS from a Postgres server that explicitly does not support TLS (such as postgres running inside docker) [[@abonander]]

-   [[#66]] Declare used features for `tokio` in `sqlx-macros` explicitly

[#66]: https://github.com/launchbadge/sqlx/issues/66

## 0.2.1 - 2020-01-16

### Fixed

-   [[#64], [#65]] Fix decoding of Rows containing NULLs in MySQL [[@danielakhterov]]

[#64]: https://github.com/launchbadge/sqlx/pull/64
[#65]: https://github.com/launchbadge/sqlx/pull/65

-   [[#55]] Use a shared tokio runtime for the `query!` macro compile-time execution (under the `runtime-tokio` feature) [[@udoprog]]

[#55]: https://github.com/launchbadge/sqlx/pull/55

## 0.2.0 - 2020-01-15

### Fixed

-   https://github.com/launchbadge/sqlx/issues/47

### Added

-   Support Tokio through an optional `runtime-tokio` feature.

-   Support SQL transactions. You may now use the `begin()` function on `Pool` or `Connection` to
    start a new SQL transaction. This returns `sqlx::Transaction` which will `ROLLBACK` on `Drop`
    or can be explicitly `COMMIT` using `commit()`.

-   Support TLS connections.

## 0.1.4 - 2020-01-11

### Fixed

-   https://github.com/launchbadge/sqlx/issues/43

-   https://github.com/launchbadge/sqlx/issues/40

### Added

-   Support for `SCRAM-SHA-256` authentication in Postgres [#37](https://github.com/launchbadge/sqlx/pull/37) [@danielakhterov](https://github.com/danielakhterov)

-   Implement `Debug` for Pool [#42](https://github.com/launchbadge/sqlx/pull/42) [@prettynatty](https://github.com/prettynatty)

## 0.1.3 - 2020-01-06

### Fixed

-   https://github.com/launchbadge/sqlx/issues/30

## 0.1.2 - 2020-01-03

### Added

-   Support for Authentication in MySQL 5+ including the newer authentication schemes now default in MySQL 8: `mysql_native_password`, `sha256_password`, and `caching_sha2_password`.

-   [`Chrono`](https://github.com/chronotope/chrono) support for MySQL was only partially implemented (was missing `NaiveTime` and `DateTime<Utc>`).

-   `Vec<u8>` (and `[u8]`) support for MySQL (`BLOB`) and Postgres (`BYTEA`).

[@abonander]: https://github.com/abonander
[@danielakhterov]: https://github.com/danielakhterov
[@mehcode]: https://github.com/mehcode
[@udoprog]: https://github.com/udoprog
[@jplatte]: https://github.com/jplatte
[@yaahc]: https://github.com/yaahc
[@freax13]: https://github.com/Freax13
[@repnop]: https://github.com/repnop
[@bmisiak]: https://github.com/bmisiak
[@oeb25]: https://github.com/oeb25
[@poiscript]: https://github.com/PoiScript
[@utter-step]: https://github.com/utter-step
[@sidred]: https://github.com/sidred
[@ace4896]: https://github.com/Ace4896
[@jamwaffles]: https://github.com/jamwaffles
[@nrjais]: https://github.com/nrjais
[@qtbeee]: https://github.com/qtbeee
[@xiaopengli89]: https://github.com/xiaopengli89
[@meh]: https://github.com/meh
[@shssoichiro]: https://github.com/shssoichiro
[@nilix007]: https://github.com/Nilix007
[@g-s-k]: https://github.com/g-s-k
[@blackwolf12333]: https://github.com/blackwolf12333
[@xyzd]: https://github.com/xyzd
[@hasali19]: https://github.com/hasali19
[@oriolmunoz]: https://github.com/OriolMunoz
[@pimeys]: https://github.com/pimeys
[@agentsim]: https://github.com/agentsim
[@meteficha]: https://github.com/meteficha
[@felipesere]: https://github.com/felipesere
[@dimtion]: https://github.com/dimtion
