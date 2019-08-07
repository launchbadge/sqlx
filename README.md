# SQLx
 
 * **Asynchronous**. Handle thousands of database connections from a single thread.
 
 * **Native**. SQLx is a pure Rust† toolkit for SQL. Where possible, drivers are written from scratch, in Rust, utilizing the modern (as of Aug. 2019) ecosystem (the `runtime` crate) for asynchronous network services development.
 
 * **Modular**. _TO BE WRITTEN_
 
 * **Fast**. _TO BE WRITTEN_
 
 * **Agnostic**. SQLx is agnostic over the database engine and can operate against a variety of database backends with the backend chosen **at compile-time** through generic constraints **or at runtime** with a slight performance loss (due to dynamic dispatch).

<sub><sup>† The SQLite driver (which does not yet exist) uses the libsqlite3 C library as SQLite is an embedded database (the only way we could be pure Rust for SQLite is by porting _all_ of SQLite to Rust).</sup></sub>

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
