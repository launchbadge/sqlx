# MSSQL (SQL Server) Support for SQLx

A complete developer guide for using SQLx with Microsoft SQL Server, built on the [Tiberius](https://github.com/prisma/tiberius) TDS driver.

---

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Feature Flags](#feature-flags)
- [Connection & Authentication](#connection--authentication)
- [Connection Pooling](#connection-pooling)
- [SSL/TLS](#ssltls)
- [Type Mappings](#type-mappings)
- [Querying](#querying)
- [Compile-Time Query Macros](#compile-time-query-macros)
- [FromRow & Derive Macros](#fromrow--derive-macros)
- [QueryBuilder](#querybuilder)
- [Transactions & Isolation Levels](#transactions--isolation-levels)
- [Migrations](#migrations)
- [Advisory Locks](#advisory-locks)
- [Bulk Insert](#bulk-insert)
- [XML Type](#xml-type)
- [Error Handling](#error-handling)
- [Any Driver Support](#any-driver-support)
- [Examples](#examples)
- [Docker & CI](#docker--ci)
- [Test Coverage](#test-coverage)

---

## Overview

Full SQL Server support has been added to SQLx, bringing feature parity with PostgreSQL, MySQL, and SQLite where applicable. The implementation provides:

- Complete type system mapping between Rust and SQL Server types
- Four authentication methods (SQL Server, Windows/NTLM, Integrated/GSSAPI, Azure AD)
- SSL/TLS with configurable modes
- Compile-time checked queries via macros
- Connection pooling with callbacks
- Runtime-polymorphic `Any` driver support
- Database migrations with `sqlx migrate`
- RAII advisory locks via `sp_getapplock`/`sp_releaseapplock`
- Bulk insert via the TDS `INSERT BULK` protocol
- Transaction isolation levels including `SNAPSHOT`
- Nested transactions via savepoints
- Testing infrastructure with Docker Compose (MSSQL 2019 & 2022)

**URL schemes:** `mssql://` and `sqlserver://`

---

## Getting Started

### Add SQLx to Your Project

SQLx requires three choices in your feature flags:

1. **Database driver** — `mssql`
2. **Async runtime** — one of `runtime-tokio` or `runtime-async-std`
3. **TLS backend** — one of `tls-native-tls`, `tls-rustls-aws-lc-rs`, `tls-rustls-ring`, or `tls-none`

```toml
[dependencies]
sqlx = { version = "0.9", features = [
    "mssql",             # SQL Server driver
    "runtime-tokio",     # async runtime (or runtime-async-std)
    "tls-native-tls",    # TLS backend (see Feature Flags for options)
] }
tokio = { version = "1", features = ["full"] }
```

> **Tip:** If you're unsure which TLS backend to pick, `tls-native-tls` is the safest default for SQL Server — it uses the platform's native TLS stack (SChannel on Windows, OpenSSL on Linux) and has the best compatibility with SQL Server's TLS implementation.

### Minimal Example

```rust
use sqlx::mssql::MssqlPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = MssqlPool::connect("mssql://sa:YourStrong!Passw0rd@localhost/master").await?;

    let row: (i32,) = sqlx::query_as("SELECT @p1")
        .bind(42i32)
        .fetch_one(&pool)
        .await?;

    println!("Got: {}", row.0);
    Ok(())
}
```

---

## Feature Flags

### Required

| Feature | Description |
|---------|-------------|
| `mssql` | Enable the MSSQL driver |

### Async Runtime (pick one)

| Feature | Description |
|---------|-------------|
| `runtime-tokio` | Use Tokio |
| `runtime-async-std` | Use async-std (via async-global-executor / smol) |

### TLS Backend (pick one)

| Feature | Description |
|---------|-------------|
| `tls-native-tls` | Platform-native TLS (recommended for SQL Server) |
| `tls-rustls-aws-lc-rs` | Rustls with AWS LC crypto |
| `tls-rustls-ring` | Rustls with ring crypto |
| `tls-none` | No TLS support |

### Type Integrations

| Feature | Description |
|---------|-------------|
| `json` | JSON type support via `serde_json` (stored as `NVARCHAR`) |
| `uuid` | `uuid::Uuid` ↔ `UNIQUEIDENTIFIER` |
| `chrono` | `chrono` datetime types |
| `time` | `time` crate datetime types |
| `rust_decimal` | `rust_decimal::Decimal` ↔ `DECIMAL`/`NUMERIC`/`MONEY` |
| `bigdecimal` | `bigdecimal::BigDecimal` ↔ `DECIMAL`/`NUMERIC`/`MONEY` |

### Authentication

| Feature | Description |
|---------|-------------|
| `winauth` | Windows/NTLM authentication |
| `integrated-auth-gssapi` | Integrated auth (Kerberos on Unix, SSPI on Windows) |

### Functionality

| Feature | Description |
|---------|-------------|
| `any` | Runtime-polymorphic `Any` driver |
| `migrate` | Database migrations |
| `offline` | Offline mode for compile-time macros (no live database needed in CI) |

### Recommended Starter Set

For most applications:

```toml
sqlx = { version = "0.9", features = [
    "mssql",
    "runtime-tokio",
    "tls-native-tls",
    "migrate",
    "json",
    "chrono",       # or "time"
    "uuid",
    "rust_decimal",
] }
```

---

## Connection & Authentication

### Connection String Format

```
mssql://[user[:password]@]host[:port][/database][?properties]
```

### Connection Options

| Option | Default | Description |
|--------|---------|-------------|
| `host` | `localhost` | Database server hostname |
| `port` | `1433` | Port number |
| `username` | `sa` | Username |
| `password` | — | Password |
| `database` | — | Database name |
| `instance` | — | SQL Server named instance |
| `app_name` | `sqlx` | Application name sent to server |
| `statement-cache-capacity` | `100` | Max cached prepared statements |
| `application_intent` | `read_write` | `read_write` or `read_only` (Always On replicas) |

### Programmatic Configuration

Use `MssqlConnectOptions` for full control over connection settings:

```rust
use sqlx::mssql::MssqlConnectOptions;

let opts = MssqlConnectOptions::new()
    .host("db.example.com")
    .port(1433)
    .username("app_user")
    .password("s3cret")
    .database("myapp")
    .app_name("my-service")
    .statement_cache_capacity(200)
    .application_intent_read_only(false);

let pool = MssqlPool::connect_with(opts).await?;
```

### URL-Based Configuration

```rust
use sqlx::mssql::MssqlPool;

let pool = MssqlPool::connect(
    "mssql://app_user:s3cret@db.example.com:1433/myapp?app_name=my-service"
).await?;
```

Both approaches are equivalent. Use `MssqlConnectOptions` when you need to build connection parameters dynamically (e.g., from environment variables or a config file).

### Authentication Methods

**1. SQL Server Auth (default)**

Standard username/password authentication.

```rust
let pool = MssqlPool::connect("mssql://sa:password@localhost/mydb").await?;
```

**2. Windows/NTLM Auth** (feature: `winauth`)

Supports `domain\user` syntax.

```rust
let opts = MssqlConnectOptions::new()
    .host("localhost")
    .windows_auth(true);
```

**3. Integrated Auth / GSSAPI** (feature: `integrated-auth-gssapi`)

Uses SSPI on Windows and Kerberos on Unix.

```rust
let opts = MssqlConnectOptions::new()
    .host("localhost")
    .integrated_auth(true);
```

**4. Azure AD Token Auth**

Pass a bearer token for Azure Active Directory authentication. This takes precedence over all other auth methods.

```rust
let opts = MssqlConnectOptions::new()
    .host("your-server.database.windows.net")
    .aad_token("eyJ0eX...");
```

---

## Connection Pooling

For production applications, always use a connection pool rather than individual connections.

### Basic Pool

```rust
use sqlx::mssql::MssqlPool;

// Simple — uses default pool settings
let pool = MssqlPool::connect("mssql://sa:password@localhost/mydb").await?;
```

### Configuring the Pool

```rust
use sqlx::mssql::{MssqlPool, MssqlPoolOptions};
use std::time::Duration;

let pool = MssqlPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(10))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)
    .connect("mssql://sa:password@localhost/mydb")
    .await?;
```

### Pool Configuration Reference

| Option | Default | Description |
|--------|---------|-------------|
| `max_connections` | `10` | Maximum number of connections in the pool |
| `min_connections` | `0` | Minimum idle connections maintained (best-effort) |
| `acquire_timeout` | `30s` | Max time to wait for a connection (includes all phases) |
| `idle_timeout` | `10min` | Close connections idle longer than this |
| `max_lifetime` | `30min` | Close connections older than this |
| `test_before_acquire` | `true` | Ping idle connections before returning them |
| `acquire_slow_threshold` | `2s` | Log a warning for acquires slower than this |

### Eager vs Lazy Connection

```rust
// connect() — opens at least one connection immediately, fails fast on bad credentials
let pool = MssqlPoolOptions::new()
    .connect("mssql://sa:password@localhost/mydb")
    .await?;

// connect_lazy() — no connections opened until first use
// Useful in tests or when the database may not be available at startup
let pool = MssqlPoolOptions::new()
    .connect_lazy("mssql://sa:password@localhost/mydb")?;
```

### Pool Callbacks

Callbacks let you run logic at key points in a connection's lifecycle:

```rust
let pool = MssqlPoolOptions::new()
    .max_connections(10)
    // Called after a new connection is established
    .after_connect(|conn, _metadata| {
        Box::pin(async move {
            // e.g., SET session options
            sqlx::query("SET ANSI_NULLS ON")
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
    })
    // Called before returning an idle connection from the pool
    .before_acquire(|conn, _metadata| {
        Box::pin(async move {
            // Return Ok(true) to use this connection
            // Return Ok(false) to close it and try another
            Ok(true)
        })
    })
    // Called when a connection is returned to the pool
    .after_release(|conn, _metadata| {
        Box::pin(async move {
            // Return Ok(true) to keep in the pool
            // Return Ok(false) to close it
            Ok(true)
        })
    })
    .connect("mssql://sa:password@localhost/mydb")
    .await?;
```

Each callback receives a `PoolConnectionMetadata` with:
- `age` — time since the connection was first opened
- `idle_for` — time the connection has been idle (only meaningful in `before_acquire`)

### Production Tuning Tips

- Set `max_connections` based on your workload and SQL Server's `max worker threads` setting. A good starting point is 2× the number of CPU cores.
- Set `min_connections` to keep a warm pool and avoid cold-start latency.
- Keep `max_lifetime` at 30 minutes or less to cycle connections and pick up DNS changes.
- Use `after_connect` to set session-level options (e.g., `SET ANSI_NULLS ON`).
- Use `test_before_acquire(true)` (the default) in production. Disable only if latency is critical and you handle stale connections at the application level.

---

## SSL/TLS

Configurable encryption modes for the TDS connection.

| Mode | Description |
|------|-------------|
| `Disabled` | No encryption |
| `LoginOnly` | Encrypt login packet only |
| `Preferred` (default) | Encrypt if server supports it |
| `Required` | Always encrypt, fail otherwise |

**Connection string parameters:**

| Parameter | Description |
|-----------|-------------|
| `sslmode` / `ssl_mode` | `disabled`, `login_only`, `preferred`, `required` |
| `encrypt` | Legacy alias: `true` = required, `false` = disabled |
| `trust_server_certificate` | Trust without validation (default: `false`) |
| `trust_server_certificate_ca` | Path to CA certificate file (`.pem`, `.crt`, `.der`) |

> **Note:** `trust_server_certificate` and `trust_server_certificate_ca` are mutually exclusive. If both are set, the CA path takes precedence.

```
mssql://sa:password@localhost/mydb?sslmode=required&trust_server_certificate=true
```

**Programmatic configuration:**

```rust
use sqlx::mssql::{MssqlConnectOptions, MssqlSslMode};

let opts = MssqlConnectOptions::new()
    .host("db.example.com")
    .ssl_mode(MssqlSslMode::Required)
    .trust_server_certificate(false)
    .trust_server_certificate_ca("/path/to/ca.pem");
```

---

## Type Mappings

### Primitive Types

| Rust Type | SQL Server Type(s) | Notes |
|-----------|-------------------|-------|
| `bool` | `BIT` | |
| `u8` | `TINYINT` | Unsigned, full range 0–255 |
| `i8` | `TINYINT` | **Only 0–127** (SQL Server TINYINT is unsigned; values 128–255 don't fit in `i8`) |
| `i16` | `SMALLINT` | |
| `i32` | `INT` | |
| `i64` | `BIGINT` | |
| `f32` | `REAL`, `FLOAT` | |
| `f64` | `REAL`, `FLOAT`, `MONEY`, `SMALLMONEY` | |
| `&str` / `String` | `NVARCHAR` | |
| `&[u8]` / `Vec<u8>` | `VARBINARY` | |

### Feature-Gated Types

#### `uuid`

| Rust Type | SQL Server Type |
|-----------|----------------|
| `uuid::Uuid` | `UNIQUEIDENTIFIER` |

#### `rust_decimal`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `rust_decimal::Decimal` | `DECIMAL`, `NUMERIC`, `MONEY`, `SMALLMONEY` |

#### `bigdecimal`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `bigdecimal::BigDecimal` | `DECIMAL`, `NUMERIC`, `MONEY` |

#### `chrono`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `chrono::NaiveDate` | `DATE` |
| `chrono::NaiveTime` | `TIME` |
| `chrono::NaiveDateTime` | `DATETIME2`, `DATETIME`, `SMALLDATETIME` |
| `chrono::DateTime<Utc>` | `DATETIME2`, `DATETIMEOFFSET` |
| `chrono::DateTime<FixedOffset>` | `DATETIMEOFFSET`, `DATETIME2` |

#### `time`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `time::Date` | `DATE` |
| `time::Time` | `TIME` |
| `time::PrimitiveDateTime` | `DATETIME2`, `DATETIME`, `SMALLDATETIME` |
| `time::OffsetDateTime` | `DATETIMEOFFSET`, `DATETIME2` |

#### `json`

| Rust Type | SQL Server Type |
|-----------|----------------|
| `serde_json::Value` / `Json<T>` | `NVARCHAR` |

> **Note:** SQL Server has no native JSON column type. JSON is stored as `NVARCHAR` text. You can still use SQL Server's built-in JSON functions (`JSON_VALUE`, `OPENJSON`, etc.) in your queries.

#### XML

| Rust Type | SQL Server Type |
|-----------|----------------|
| `MssqlXml` | `XML` |

### Nullable Types

All types above support `Option<T>` for nullable columns.

### Runtime Type Inspection

Use `MssqlTypeInfo` to inspect column types at runtime:

```rust
use sqlx::TypeInfo;

let statement = conn.prepare("SELECT id, name FROM users".into_sql_str()).await?;
assert_eq!(statement.column(0).type_info().name(), "BIGINT");
assert_eq!(statement.column(1).type_info().name(), "NVARCHAR");
```

---

## Querying

MSSQL uses `@p1`, `@p2`, `@p3`, ... as parameter placeholders (not `$1` or `?`).

### Basic Queries

```rust
use sqlx::Row;

// Execute a statement (INSERT, UPDATE, DELETE)
let result = sqlx::query("UPDATE users SET active = 1 WHERE id = @p1")
    .bind(42i32)
    .execute(&pool)
    .await?;
println!("Rows affected: {}", result.rows_affected());

// Fetch a single row
let row = sqlx::query("SELECT id, name FROM users WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;
let name: String = row.get("name");

// Fetch all rows
let rows = sqlx::query("SELECT id, name FROM users")
    .fetch_all(&pool)
    .await?;

// Fetch optional (returns None if no rows)
let maybe_row = sqlx::query("SELECT id FROM users WHERE email = @p1")
    .bind("alice@example.com")
    .fetch_optional(&pool)
    .await?;
```

### Typed Queries with `query_as`

```rust
let user: (i32, String) = sqlx::query_as("SELECT id, name FROM users WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;

// Or with a named struct (see FromRow section)
let user: User = sqlx::query_as("SELECT id, name FROM users WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;
```

### Scalar Queries

```rust
let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await?;
```

### Streaming with `fetch`

For large result sets, use `fetch` to stream rows without loading them all into memory:

```rust
use futures::TryStreamExt;

let mut stream = sqlx::query("SELECT id, name FROM users")
    .fetch(&pool);

while let Some(row) = stream.try_next().await? {
    let id: i32 = row.get("id");
    // process row...
}
```

### Row Access

```rust
use sqlx::Row;

let row = sqlx::query("SELECT id, name FROM users")
    .fetch_one(&pool)
    .await?;

// By column name
let name: String = row.get("name");

// By column index (0-based)
let id: i32 = row.get(0);

// Fallible access (returns Result)
let name: String = row.try_get("name")?;
```

### Custom Row Mapping

```rust
let value = sqlx::query("SELECT 1 + @p1")
    .bind(5_i32)
    .try_map(|row: MssqlRow| row.try_get::<i32, _>(0))
    .fetch_one(&pool)
    .await?;
```

### OUTPUT INSERTED (MSSQL's RETURNING)

SQL Server does not support the `RETURNING` clause. Use `OUTPUT INSERTED` instead to get values from inserted/updated rows:

```rust
// Get the auto-generated ID after INSERT
let id: i64 = sqlx::query_scalar(
    "INSERT INTO users (name) OUTPUT INSERTED.id VALUES (@p1)"
)
    .bind("Alice")
    .fetch_one(&pool)
    .await?;

// Get multiple columns
let row = sqlx::query(
    "INSERT INTO users (name, email) OUTPUT INSERTED.id, INSERTED.created_at VALUES (@p1, @p2)"
)
    .bind("Alice")
    .bind("alice@example.com")
    .fetch_one(&pool)
    .await?;
```

### Calling Stored Procedures

Use `EXEC` to call stored procedures:

```rust
let rows = sqlx::query("EXEC GetUsersByRole @p1")
    .bind("admin")
    .fetch_all(&pool)
    .await?;

// With output parameters, use a query that captures results
let result: (i32,) = sqlx::query_as("EXEC CountUsers")
    .fetch_one(&pool)
    .await?;
```

---

## Compile-Time Query Macros

The standard SQLx macros work with MSSQL when `DATABASE_URL` is set to an `mssql://` connection string:

```rust
// Compile-time checked query
let row = sqlx::query!("SELECT @p1 AS value", 42i32)
    .fetch_one(&pool)
    .await?;

// With custom return type
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
}

let user = sqlx::query_as!(User, "SELECT id, name FROM users WHERE id = @p1", 1i32)
    .fetch_one(&pool)
    .await?;

// Scalar queries
let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await?;
```

### Offline Mode

For CI builds without a live database, use offline mode:

```bash
# Generate query metadata (run with DATABASE_URL set)
cargo sqlx prepare

# This creates a .sqlx/ directory with cached query metadata.
# Commit this directory to version control.
```

Then build without a database:

```bash
SQLX_OFFLINE=true cargo build
```

Enable the `offline` feature flag to use this capability.

---

## FromRow & Derive Macros

### Basic FromRow

Map query results directly to a struct:

```rust
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
    email: Option<String>,
}

let user: User = sqlx::query_as("SELECT id, name, email FROM users WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;
```

### Enum Types with `#[derive(Type)]`

**Integer-repr enums** map to SQL integer columns:

```rust
#[derive(sqlx::Type, Debug, PartialEq)]
#[repr(i32)]
enum Status {
    Active = 1,
    Inactive = 0,
    Banned = -1,
}

// Works with INT columns
let status: Status = sqlx::query_scalar("SELECT status FROM users WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;
```

**Transparent wrappers** create newtypes over existing SQL types:

```rust
#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(transparent)]
struct UserId(i64);

let id: UserId = sqlx::query_scalar("SELECT id FROM users WHERE id = @p1")
    .bind(1i64)
    .fetch_one(&pool)
    .await?;
```

### Combining FromRow and Type

```rust
#[derive(sqlx::Type, Debug, PartialEq)]
#[repr(i16)]
enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(sqlx::FromRow, Debug)]
struct Task {
    id: i32,
    title: String,
    priority: Priority,
}

let task: Task = sqlx::query_as("SELECT id, title, priority FROM tasks WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&pool)
    .await?;
```

---

## QueryBuilder

`QueryBuilder` generates MSSQL-style parameter placeholders (`@p1`, `@p2`, ...) automatically:

```rust
use sqlx::QueryBuilder;
use sqlx::mssql::Mssql;

let mut qb: QueryBuilder<Mssql> = QueryBuilder::new("SELECT * FROM users WHERE ");
qb.push("name = ").push_bind("Alice");
qb.push(" AND age > ").push_bind(21i32);
// Produces: SELECT * FROM users WHERE name = @p1 AND age > @p2

let users = qb.build_query_as::<User>()
    .fetch_all(&pool)
    .await?;
```

### Dynamic WHERE Clauses

```rust
let mut qb: QueryBuilder<Mssql> = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

if let Some(name) = filter_name {
    qb.push(" AND name = ").push_bind(name);
}
if let Some(min_age) = filter_min_age {
    qb.push(" AND age >= ").push_bind(min_age);
}

let results = qb.build_query_as::<User>().fetch_all(&pool).await?;
```

### Reset and Rebuild

```rust
let mut qb: QueryBuilder<Mssql> = QueryBuilder::new("SELECT * FROM users");
let query = qb.build();
// ... use query ...

// Reset to build a new query with the same builder
qb.reset();
qb.push("SELECT COUNT(*) FROM users");
let count_query = qb.build();
```

---

## Transactions & Isolation Levels

### Basic Transactions

```rust
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO users (name) VALUES (@p1)")
    .bind("Alice")
    .execute(&mut *tx)
    .await?;

sqlx::query("INSERT INTO audit_log (action) VALUES (@p1)")
    .bind("user_created")
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
// Or: tx.rollback().await?;
```

### Nested Transactions (Savepoints)

Calling `begin()` on an existing transaction creates a savepoint:

```rust
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO users (id, name) VALUES (@p1, @p2)")
    .bind(1i32)
    .bind("Alice")
    .execute(&mut *tx)
    .await?;

// Nested transaction — creates a savepoint
let mut savepoint = tx.begin().await?;

sqlx::query("INSERT INTO users (id, name) VALUES (@p1, @p2)")
    .bind(2i32)
    .bind("Bob")
    .execute(&mut *savepoint)
    .await?;

// Roll back only the inner transaction
savepoint.rollback().await?;
// Bob's insert is undone, but Alice's remains

tx.commit().await?;
// Alice is committed, Bob is not
```

### Isolation Levels

| Level | Description |
|-------|-------------|
| `ReadUncommitted` | Dirty reads allowed |
| `ReadCommitted` | Default SQL Server isolation |
| `RepeatableRead` | Prevents non-repeatable reads |
| `Snapshot` | Row versioning-based isolation |
| `Serializable` | Strictest isolation |

> **Important:** `begin_with_isolation` is a method on `MssqlConnection`, not on `Pool`. You must acquire a connection first:

```rust
use sqlx::mssql::MssqlIsolationLevel;

let mut conn = pool.acquire().await?;
let mut tx = conn
    .begin_with_isolation(MssqlIsolationLevel::Snapshot)
    .await?;

sqlx::query("SELECT * FROM accounts WHERE id = @p1")
    .bind(1i32)
    .fetch_one(&mut *tx)
    .await?;

tx.commit().await?;
```

> **Note:** `Snapshot` isolation requires the database to have `ALLOW_SNAPSHOT_ISOLATION` enabled:
> ```sql
> ALTER DATABASE [mydb] SET ALLOW_SNAPSHOT_ISOLATION ON;
> ```

---

## Migrations

MSSQL supports the full `sqlx migrate` workflow.

```bash
# Create a new migration
sqlx migrate add create_users_table

# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert
```

**Programmatic usage:**

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

**Database lifecycle:**

- `create_database(url)` — Creates a database via `CREATE DATABASE [name]`
- `database_exists(url)` — Checks existence via `DB_ID()`
- `drop_database(url)` — Drops with `ALTER DATABASE SET SINGLE_USER WITH ROLLBACK IMMEDIATE` for cleanup

**No-transaction migrations** are supported for DDL operations that cannot run inside a transaction.

Migration files use standard SQL Server syntax. Use bracket-quoted identifiers (`[schema].[table]`) for schema-qualified objects.

---

## Advisory Locks

Application-level named locks using SQL Server's `sp_getapplock` and `sp_releaseapplock`, with an RAII guard pattern.

### Lock Modes

| Mode | Compatible With |
|------|----------------|
| `Shared` | Shared, Update |
| `Update` | Shared only |
| `Exclusive` (default) | None |

### Usage

```rust
use sqlx::mssql::{MssqlAdvisoryLock, MssqlAdvisoryLockMode};

// Create an exclusive lock
let lock = MssqlAdvisoryLock::new("my_resource");

// Or with a specific mode
let lock = MssqlAdvisoryLock::with_mode("my_resource", MssqlAdvisoryLockMode::Shared);

// RAII guard — acquire and release
let guard = lock.acquire_guard(&mut conn).await?;
// ... do work while lock is held ...
let conn = guard.release_now().await?; // explicit release

// Non-blocking attempt
match lock.try_acquire_guard(&mut conn).await? {
    either::Either::Left(guard) => {
        // Lock acquired
        let conn = guard.release_now().await?;
    }
    either::Either::Right(conn) => {
        // Lock not available
    }
}

// Manual acquire/release (without guard)
lock.acquire(&mut conn).await?;
// ... do work ...
lock.release(&mut conn).await?;
```

> **Warning:** Unlike PostgreSQL advisory locks, MSSQL advisory lock guards do **NOT** auto-release on drop. If you drop the guard without calling `release_now()` or `leak()`, a warning is logged and the lock remains held until the connection is closed or returned to the pool. Always call `release_now()` explicitly.

---

## Bulk Insert

High-performance data loading via the TDS `INSERT BULK` protocol. The target table must already exist.

```rust
use sqlx::mssql::IntoRow;

let mut bulk = conn.bulk_insert("my_table").await?;

bulk.send(("Alice", 30_i32).into_row()).await?;
bulk.send(("Bob", 25_i32).into_row()).await?;
bulk.send(("Carol", 28_i32).into_row()).await?;

let rows_affected = bulk.finalize().await?;
assert_eq!(rows_affected, 3);
```

> **Important:** You **must** call `finalize()` to flush buffered data. If the `MssqlBulkInsert` is dropped without calling `finalize()`, buffered rows are lost.

Tuple elements map to table columns in order. Tuples up to **10 elements** are supported via `tiberius::IntoRow`.

---

## XML Type

A dedicated `MssqlXml` wrapper type distinguishes XML columns from regular strings.

```rust
use sqlx::mssql::MssqlXml;

let xml = MssqlXml::from("<root><item>hello</item></root>".to_string());

sqlx::query("INSERT INTO docs (content) VALUES (@p1)")
    .bind(&xml)
    .execute(&pool)
    .await?;

let result: MssqlXml = sqlx::query_scalar("SELECT content FROM docs")
    .fetch_one(&pool)
    .await?;
```

---

## Error Handling

### Error Types

All SQLx operations return `sqlx::Error`. For database-specific errors, downcast to `MssqlDatabaseError`:

```rust
use sqlx::error::ErrorKind;

let result = sqlx::query("INSERT INTO users (id, name) VALUES (@p1, @p2)")
    .bind(1i32)
    .bind("Alice")
    .execute(&pool)
    .await;

match result {
    Ok(r) => println!("Inserted {} rows", r.rows_affected()),
    Err(sqlx::Error::Database(db_err)) => {
        // Classify the error
        match db_err.kind() {
            ErrorKind::UniqueViolation => {
                println!("Duplicate key: {}", db_err.message());
            }
            ErrorKind::ForeignKeyViolation => {
                println!("Foreign key constraint failed");
            }
            ErrorKind::NotNullViolation => {
                println!("Required field is null");
            }
            ErrorKind::CheckViolation => {
                println!("Check constraint failed");
            }
            _ => {
                println!("Database error: {}", db_err.message());
            }
        }
    }
    Err(e) => println!("Other error: {}", e),
}
```

### MssqlDatabaseError Fields

When you need SQL Server-specific error details, downcast further:

```rust
use sqlx::mssql::MssqlDatabaseError;

if let sqlx::Error::Database(db_err) = &err {
    if let Some(mssql_err) = db_err.try_downcast_ref::<MssqlDatabaseError>() {
        println!("Error number: {}", mssql_err.number());     // SQL Server error number
        println!("State: {}", mssql_err.state());              // Error state
        println!("Class: {}", mssql_err.class());              // Severity class
        println!("Message: {}", mssql_err.message());          // Error message
        println!("Server: {:?}", mssql_err.server());          // Server name (Option)
        println!("Procedure: {:?}", mssql_err.procedure());    // Stored procedure name (Option)
    }
}
```

### ErrorKind Mapping

| SQL Server Error Number | ErrorKind |
|------------------------|-----------|
| 2601, 2627 | `UniqueViolation` |
| 547 | `ForeignKeyViolation` |
| 515 | `NotNullViolation` |
| 2628 | `CheckViolation` |
| All others | `Other` |

### Connection Recovery

Connections remain usable after query errors:

```rust
// This query fails
let result = sqlx::query("SELECT * FROM nonexistent_table")
    .execute(&mut conn)
    .await;
assert!(result.is_err());

// Connection is still valid
let val: (i32,) = sqlx::query_as("SELECT 42")
    .fetch_one(&mut conn)
    .await?;
```

---

## Any Driver Support

MSSQL is fully integrated with the `Any` runtime-polymorphic driver, enabled via the `any` feature flag.

```rust
use sqlx::any::AnyPool;

// Connects to whichever database the URL points to
let pool = AnyPool::connect("mssql://sa:password@localhost/mydb").await?;

let rows = sqlx::query("SELECT 1 + 1 AS result")
    .fetch_all(&pool)
    .await?;
```

All standard operations work through `Any`: queries, transactions, ping, close, and prepared statements.

---

## Examples

A full CRUD Todo application is available at `examples/mssql/todos/`, demonstrating connection pooling, migrations, query execution, and error handling.

---

## Docker & CI

### Docker Compose

The test suite includes Docker Compose configurations for MSSQL 2019 and 2022:

```bash
docker compose -f tests/docker-compose.yml up mssql_2022 -d
```

**Services:**

| Service | Image | Port |
|---------|-------|------|
| `mssql_2022` | `mcr.microsoft.com/mssql/server:2022-latest` | 1433 |
| `mssql_2019` | `mcr.microsoft.com/mssql/server:2019-latest` | 1433 |

### CI Matrix

The GitHub Actions workflow tests across:

- **MSSQL versions:** 2019, 2022
- **Async runtimes:** tokio, async-global-executor, smol
- **TLS backends:** native-tls, rustls-aws-lc-rs, rustls-ring, none

---

## Test Coverage

Comprehensive test suite in `tests/mssql/`:

| Area | File | What's Tested |
|------|------|---------------|
| Core queries | `mssql.rs` | Connections, SELECT, INSERT, parameters, large result sets, error handling |
| Type round-trips | `types.rs` | All primitive and feature-gated types with boundary values, NULLs, Unicode, large data |
| Test attribute | `test-attr.rs` | `#[sqlx_macros::test]` macro with automatic test DB setup |
| Isolation levels | `isolation-level.rs` | All five isolation level configurations |
| Advisory locks | `advisory-lock.rs` | Acquire, release, guard pattern, all lock modes |
| Bulk insert | `bulk-insert.rs` | High-performance loading, multi-row operations |
| Derives | `derives.rs` | `#[derive(FromRow)]`, custom field mappings |
| Query builder | `query_builder.rs` | Dynamic query construction, parameter handling |
| Error handling | `error.rs` | Database error inspection, error details |
| Compile-time macros | `tests/mssql-macros/` | Online and offline macro verification |
