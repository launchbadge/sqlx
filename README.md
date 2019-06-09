# Mason
_Asynchronous and expressive database client in pure Rust_

This is an experiment being worked on in stages. The first stage
will be a very low-level, generic database driver (hopefully) capable of basic execution of
simple queries.

## Usage

What follows is _experimental_ usage (for thinking on API design) that is not currently implemented.

```rust
#![feature(async_await)]

use mason::pg::Connection;

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    // this will likely be something like eventually:
    //  mason::Connection::<Pg>::establish(...)

    let mut conn = Connection::establish(ConnectOptions::new().user("postgres")).await?;
    // or: Connection::establish("postgres://postgres@localhost/").await?;
    // or: ConnectOptions::new().user("postgres").establish().await?;

    // Execute a "simple" query. Can consist of N statements separated by semicolons.
    // No results are returned.

    conn.execute("CREATE TABLE IF NOT EXISTS users ( id UUID PRIMARY KEY, name TEXT NOT NULL );")
        .await?;

    // prepare() -> Statement
    //  - A `Statement` can be cached and re-bound later for improved performance
    conn.prepare("SELECT id FROM users WHERE name ilike $1")
        // bind() -> Cursor (named [Cursor] in mysql or sqlite but [Portal] in postgres)
        .bind(&["bob"])
        // execute() -> u64
        //  - execute may be used instead of fetch to ignore all results and only
        //    return the "affected" rows
        // fetch() -> Stream<Item = Row>
        .fetch()
        .collect::<Vec<Row>>()
        .await?;

    // Close is not strictly needed but this makes sure any pending writes to the connection
    // are flushed and gracefully closes the connection

    conn.close().await?;

    Ok(())
}
```

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
