<h1 align="center">SQLx</h1>
<div align="center">
 <strong>
   🧰 The Rust SQL Toolkit
 </strong>
</div>

<br />

<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/v/sqlx.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Discord -->
  <a href="https://discord.gg/uuruzJ7">
    <img src="https://img.shields.io/discord/665528275556106240?style=flat-square" alt="chat" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/d/sqlx.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/sqlx">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- Github Actions -->
  <img src="https://img.shields.io/github/workflow/status/launchbadge/sqlx/Rust?style=flat-square" alt="actions status" />
</div>

<br />

<div align="center">
  <sub>Built with ❤️ by <a href="https://launchbadge.com">The LaunchBadge team</a>
</div>

<br />

SQLx is an async, pure Rust SQL crate featuring compile-time checked queries without a DSL.

 * **Truly Asynchronous**. Built from the ground-up using async/await for maximum concurrency.

 * **Type-safe SQL** (if you want it) without DSLs. Use the `query!()` macro to check your SQL and bind parameters at 
 compile time. (You can still use dynamic SQL queries if you like.)

 * **Pure Rust**. The Postgres and MySQL/MariaDB drivers are written in pure Rust using **zero** unsafe code.
 
 * **Runtime Agnostic**. Works on [async-std](https://crates.io/crates/async-std) or [tokio](https://crates.io/crates/tokio) with the `runtime-async-std` or `runtime-tokio` cargo feature flag.

## Install

**async-std**

```toml
# Cargo.toml
[dependencies]
sqlx = "0.2"
```

**tokio**

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.2", default-features = false, features = [ "runtime-tokio", "macros" ] }
```

#### Cargo Feature Flags

 * `runtime-async-std` (on by default): Use the `async-std` runtime.
 
 * `runtime-tokio`: Use the `tokio` runtime. Mutually exclusive with the `runtime-async-std` feature.
 
 * `postgres`: Add support for the Postgres database server.
 
 * `mysql`: Add support for the MySQL (and MariaDB) database server.
 
 * `uuid`: Add support for UUID (in Postgres).
 
 * `chrono`: Add support for date and time types from `chrono`.
 
 * `tls`: Add support for TLS connections.

## Examples

#### Connect

It is a very good idea to always create a connection pool at the beginning of your application and then share that.

```rust
// Postgres
let pool = sqlx::PgPool::new("postgres://localhost/database").await?;
```

#### Dynamic

The `sqlx::query` function provides general-purpose prepared statement execution. 
The result is an implementation of the `Row` trait. Values can be efficiently accessed by index or name.

```rust
let row = sqlx::query("SELECT is_active FROM users WHERE id = ?")
    .bind(some_user_id)
    .fetch_one(&mut &pool)
    .await?;
    
let is_active: bool = row.get("is_active");
```

#### Static

The `sqlx::query!` macro prepares the SQL query at compile time and interprets the result in order to constrain input types and 
infer output types. The result of `query!` is an anonymous struct (or named tuple).

```rust
let countries = sqlx::query!(
        "SELECT country, COUNT(*) FROM users GROUP BY country WHERE organization = ?", 
        organization
    )
    .fetch(&mut &pool) // -> impl Stream<Item = { country: String, count: i64 }>
    .map_ok(|rec| (rec.country, rec.count))
    .try_collect::<HashMap<_>>() // -> HashMap<String, i64>
    .await?;
```

For this mode, the `DATABASE_URL` environment variable must be set at build time to a database which it can prepare queries
against; the database does not have to contain any data but must be the same kind (MySQL, Postgres, etc.) and have the same schema as the database you will be connecting to at runtime. For convenience, you can use [a `.env` file](https://github.com/dotenv-rs/dotenv#examples) to set `DATABASE_URL` so that you don't have to pass it every time:

```
DATABASE_URL=mysql://localhost/my_database
```

See the beginnings of a [RealWorld](https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication) implementation in [examples/realworld-postgres](./examples/realworld-postgres).

## Safety

This crate uses `#[forbid(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
