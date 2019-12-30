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

SQLx is a modern SQL client built from the ground up for Rust, in Rust.

 * **Truly Asynchronous**. Built from the ground-up using [async-std] using async streams for maximum concurrency.

 * **Type-safe SQL** (if you want it) without DSLs. Use the `query!()` macro to check your SQL and bind parameters at 
 compile time. (You can still use dynamic SQL queries if you like.)

 * **Pure Rust**. The Postgres and MySQL/MariaDB drivers are written in pure Rust using **zero** unsafe code.

[async-std]: https://github.com/async-rs/async-std
## Examples

The `sqlx::query` function provides general-purpose prepared statement execution. 
The result is an implementation of the `Row` trait. Values can be efficiently accessed by index or name.

```rust
let row = sqlx::query("SELECT is_active FROM users WHERE id = ?")
    .bind(some_user_id)
    .fetch_one(&mut conn)
    .await?;
    
let is_active: bool = row.get("is_active");
```

The `sqlx::query!` macro prepares the SQL query and interprets the result in order to constrain input types and 
infer output types. The result of `query!` is an anoymous struct (or named tuple).

```rust
let countries = sqlx::query!(
        "SELECT country, COUNT(*) FROM users GROUP BY country WHERE organization = ?", 
        organization
    )
    .fetch(&mut conn) // -> impl Stream<Item = { country: String, count: i64 }>
    .map_ok(|rec| (rec.country, rec.count))
    .collect::<HashMap<_>>() // -> HashMap<String, i64>
    .await?;
```

See the beginnings of a [RealWorld](https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication) implementation in [examples/realworld-postgres](./examples/realworld-postgres).

## Safety

This crate uses `#[deny(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

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
