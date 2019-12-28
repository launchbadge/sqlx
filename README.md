<h1 align="center">SQLx</h1>
<div align="center">
 <strong>
   🧰 The Rust SQL Toolkit
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/v/sqlx.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sqlx">
    <img src="https://img.shields.io/crates/d/sqlx.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/sqlx">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<br />

<div align="center">
  <sub>Built with ❤️ by <a href="https://launchbadge.com">The LaunchBadge team</a>
</div>

<br />

SQLx is a modern SQL client built from the ground up for Rust, in Rust.

 * **Asynchronous**.

 * **Native**. SQLx is a pure Rust toolkit for SQL. Where possible, drivers are written from scratch, in Rust, utilizing the modern ecosystem for asynchronous network services development.

 * **Type-safe**. SQLx is built upon the novel idea of preparing SQL statements before or duing compilation to provide strong type safety while not getting in your way with a custom DSL. 

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

See the beginngins of a [RealWorld](https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication) implementation in [examples/realworld-postgres](./examples/realworld-postgres).

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
