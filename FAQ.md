SQLx Frequently Asked Questions
===============================

### What database versions does SQLx support?

This is a difficult question to answer because it depends on which features of the databases are used and when those features were introduced. SQL databases tend to be very strongly backwards-compatible so it's likely that SQLx will work with some very old versions. 

TLS support is one of the features that ages most quickly with databases, since old SSL/TLS versions are deprecated over time as they become insecure due to weaknesses being discovered; this is especially important to consider when using RusTLS, as it only supports the latest TLS version for security reasons (see the question below mentioning RusTLS for details).

As a rule, however, we only officially support the range of versions for each database that are still actively maintained, and will drop support for versions as they reach their end-of-life.

* Postgres has a page to track these versions and give their end-of-life dates: https://www.postgresql.org/support/versioning/
* MariaDB has a similar list here (though it doesn't show the dates at which old versions were EOL'd): https://mariadb.com/kb/en/mariadb-server-release-dates/
* MySQL's equivalent page is more concerned with what platforms are supported by the newest and oldest maintained versions: https://www.mysql.com/support/supportedplatforms/database.html
    * However, its Wikipedia page helpfully tracks its versions and their announced EOL dates: https://en.wikipedia.org/wiki/MySQL#Release_history
* SQLite is easy as only SQLite 3 is supported and the current version depends on the version of the `libsqlite3-sys` crate being used.

For each database and where applicable, we test against the latest and oldest versions that we intend to support. You can see the current versions being tested against by looking at our CI config: https://github.com/launchbadge/sqlx/blob/main/.github/workflows/sqlx.yml#L168

-------------------------------------------------------------------
### What versions of Rust does SQLx support? What is SQLx's MSRV\*?

Officially, we will only ever support the latest stable version of Rust. 
It's just not something we consider to be worth keeping track of, given the current state of tooling.

Cargo does support a [`rust-version`] field now in the package manifest, however that will only ensure
that the user is using a `rustc` of that version or greater; it does not enforce that the crate's code is
actually compatible with that version of Rust. That would need to be manually enforced and checked with CI,
and we have enough CI passes already.

In practice, we tend not to reach for language or library features that are *too* new, either because
we don't need them or we just worked around their absence. There are language features we're waiting to be
stabilized that we want to use (Generic Associated Types and Async Traits, to name a couple) which we will
be utilizing when they're available, but that will be a major refactor with breaking API changes 
which will then coincide with a major release.

Thus, it is likely that a given SQLx release _will_ work with older versions of Rust. However,
we make no guarantees about which exact versions it will work with besides the latest stable version,
and we don't factor MSRV bumps into our semantic versioning.

\*Minimum Supported Rust Version

[`rust-version`]: https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-rust-version-field

----------------------------------------------------------------
### I'm getting `HandshakeFailure` or `CorruptMessage` when trying to connect to a server over TLS using RusTLS. What gives?

To encourage good security practices and limit cruft, RusTLS does not support older versions of TLS or cryptographic algorithms 
that are considered insecure. `HandshakeFailure` is a normal error returned when RusTLS and the server cannot agree on parameters for
a secure connection. 

Check the supported TLS versions for the database server version you're running. If it does not support TLS 1.2 or greater, then
you likely will not be able to connect to it with RusTLS.

The ideal solution, of course, is to upgrade your database server to a version that supports at least TLS 1.2.  

* MySQL: [has supported TLS 1.2 since 5.6.46](https://dev.mysql.com/doc/refman/5.6/en/encrypted-connection-protocols-ciphers.html#encrypted-connection-supported-protocols). 
* PostgreSQL: depends on the system OpenSSL version.
* MSSQL: TLS is not supported yet.

If you're running a third-party database that talks one of these protocols, consult its documentation for supported TLS versions.

If you're stuck on an outdated version, which is unfortunate but tends to happen for one reason or another, try switching to the corresponding
`runtime-<tokio, async-std, actix>-native-tls` feature for SQLx. That will use the system APIs for TLS which tend to have much wider support.
See [the `native-tls` crate docs](https://docs.rs/native-tls/latest/native_tls/) for details.

The `CorruptMessage` error occurs in similar situations and many users have had success with switching to `-native-tls` to get around it.
However, if you do encounter this error, please try to capture a Wireshark or `tcpdump` trace of the TLS handshake as the RusTLS folks are interested
in covering cases that trigger this (as it might indicate a protocol handling bug or the server is doing something non-standard): 
https://github.com/rustls/rustls/issues/893

----------------------------------------------------------------
### How can I do a `SELECT ... WHERE foo IN (...)` query?


In the future SQLx will support binding arrays as a comma-separated list for every database,
but unfortunately there's no general solution for that currently in SQLx itself.
You would need to manually generate the query, at which point it
cannot be used with the macros.

However, **in Postgres** you can work around this limitation by binding the arrays directly and using `= ANY()`:

```rust
let db: PgPool = /* ... */;
let foo_ids: Vec<i64> = vec![/* ... */];

let foos = sqlx::query!(
    "SELECT * FROM foo WHERE id = ANY($1)",
    // a bug of the parameter typechecking code requires all array parameters to be slices
    &foo_ids[..]
)
    .fetch_all(&db)
    .await?;
```

Even when SQLx gains generic placeholder expansion for arrays, this will still be the optimal way to do it for Postgres,
as comma-expansion means each possible length of the array generates a different query 
(and represents a combinatorial explosion if more than one array is used).

Note that you can use any operator that returns a boolean, but beware that `!= ANY($1)` is **not equivalent** to `NOT IN (...)` as it effectively works like this:

`lhs != ANY(rhs) -> false OR lhs != rhs[0] OR lhs != rhs[1] OR ... lhs != rhs[length(rhs) - 1]`

The equivalent of `NOT IN (...)` would be `!= ALL($1)`:

`lhs != ALL(rhs) -> true AND lhs != rhs[0] AND lhs != rhs[1] AND ... lhs != rhs[length(rhs) - 1]`

Note that `ANY` using any operator and passed an empty array will return `false`, thus the leading `false OR ...`.  
Meanwhile, `ALL` with any operator and passed an empty array will return `true`, thus the leading `true AND ...`.

See also: [Postgres Manual, Section 9.24: Row and Array Comparisons](https://www.postgresql.org/docs/current/functions-comparisons.html)

-----
### How can I bind an array to a `VALUES()` clause? How can I do bulk inserts?

Like the above, SQLx currently does not support this in the general case right now but will in the future.

However, **Postgres** also has a feature to save the day here! You can pass an array to `UNNEST()` and
it will treat it as a temporary table:

```rust
let foo_texts: Vec<String> = vec![/* ... */];

sqlx::query!(
    // because `UNNEST()` is a generic function, Postgres needs the cast on the parameter here
    // in order to know what type to expect there when preparing the query
    "INSERT INTO foo(text_column) SELECT * FROM UNNEST($1::text[])",
    &foo_texts[..]
)
    .execute(&db)
    .await?; 
```

`UNNEST()` can also take more than one array, in which case it'll treat each array as a column in the temporary table:

```rust
// this solution currently requires each column to be its own vector
// in the future we're aiming to allow binding iterators directly as arrays
// so you can take a vector of structs and bind iterators mapping to each field
let foo_texts: Vec<String> = vec![/* ... */];
let foo_bools: Vec<bool> = vec![/* ... */];
let foo_ints: Vec<i64> = vec![/* ... */];

sqlx::query!(
    "
        INSERT INTO foo(text_column, bool_column, int_column) 
        SELECT * FROM UNNEST($1::text[], $2::bool[], $3::int8[])
    ",
    &foo_texts[..],
    &foo_bools[..],
    &foo_ints[..]
)
    .execute(&db)
    .await?;
```

Again, even with comma-expanded lists in the future this will likely still be the most performant way to run bulk inserts
with Postgres--at least until we get around to implementing an interface for `COPY FROM STDIN`, though
this solution with `UNNEST()` will still be more flexible as you can use it in queries that are more complex
than just inserting into a table.

Note that if some vectors are shorter than others, `UNNEST` will fill the corresponding columns with  `NULL`s
to match the longest vector.

For example, if `foo_texts` is length 5, `foo_bools` is length 4, `foo_ints` is length 3, the resulting table will
look like this:

| Row # | `text_column`  | `bool_column`  | `int_column`  |
| ----- | -------------- | -------------- | ------------- |
| 1     | `foo_texts[0]` | `foo_bools[0]` | `foo_ints[0]` |
| 2     | `foo_texts[1]` | `foo_bools[1]` | `foo_ints[1]` |
| 3     | `foo_texts[2]` | `foo_bools[2]` | `foo_ints[2]` |
| 4     | `foo_texts[3]` | `foo_bools[3]` | `NULL`        |
| 5     | `foo_texts[4]` | `NULL`         | `NULL`        |

See Also:
* [Postgres Manual, Section 7.2.1.4: Table Functions](https://www.postgresql.org/docs/current/queries-table-expressions.html#QUERIES-TABLEFUNCTIONS)
* [Postgres Manual, Section 9.19: Array Functions and Operators](https://www.postgresql.org/docs/current/functions-array.html)

----
### How do I compile with the macros without needing a database, e.g. in CI?

The macros support an offline mode which saves data for existing queries to a JSON file,
so the macros can just read that file instead of talking to a database.

See the following:

* [the docs for `query!()`](https://docs.rs/sqlx/0.5.5/sqlx/macro.query.html#offline-mode-requires-the-offline-feature)
* [the README for `sqlx-cli`](sqlx-cli/README.md#enable-building-in-offline-mode-with-query)

To keep `sqlx-data.json` up-to-date you need to run `cargo sqlx prepare` before every commit that
adds or changes a query; you can do this with a Git pre-commit hook:

```shell
$ echo "cargo sqlx prepare > /dev/null 2>&1; git add sqlx-data.json > /dev/null" > .git/hooks/pre-commit 
```

Note that this may make committing take some time as it'll cause your project to be recompiled, and
as an ergonomic choice it does _not_ block committing if `cargo sqlx prepare` fails.

We're working on a way for the macros to save their data to the filesystem automatically which should be part of SQLx 0.6,
so your pre-commit hook would then just need to stage the changed files.

----

### How do the query macros work under the hood?

The macros work by talking to the database at compile time. When a(n) SQL client asks to create a prepared statement 
from a query string, the response from the server typically includes information about the following:

* the number of bind parameters, and their expected types if the database is capable of inferring that
* the number, names and types of result columns, as well as the original table and columns names before aliasing 
  
In MySQL/MariaDB, we also get boolean flag signaling if a column is `NOT NULL`, however 
in Postgres and SQLite, we need to do a bit more work to determine whether a column can be `NULL` or not.

After preparing, the Postgres driver will first look up the result columns in their source table and check if they have 
a `NOT NULL` constraint. Then, it will execute `EXPLAIN (VERBOSE, FORMAT JSON) <your query>` to determine which columns 
come from half-open joins (LEFT and RIGHT joins), which makes a normally `NOT NULL` column nullable. Since the
`EXPLAIN VERBOSE` format is not stable or completely documented, this inference isn't perfect. However, it does err on
the side of producing false-positives (marking a column nullable when it's `NOT NULL`) to avoid errors at runtime.

If you do encounter false-positives please feel free to open an issue; make sure to include your query, any relevant
schema as well as the output of `EXPLAIN (VERBOSE, FORMAT JSON) <your query>` which will make this easier to debug.

The SQLite driver will pull the bytecode of the prepared statement and step through it to find any instructions
that produce a null value for any column in the output.

---
### Why can't SQLx just look at my database schema/migrations and parse the SQL itself?

Take a moment and think of the effort that would be required to do that.

To implement this for a single database driver, SQLx would need to:

* know how to parse SQL, and not just standard SQL but the specific dialect of that particular database
* know how to analyze and typecheck SQL queries in the context of the original schema
* if inferring schema from migrations it would need to simulate all the schema-changing effects of those migrations

This is effectively reimplementing a good chunk of the database server's frontend, 

_and_ maintaining and ensuring correctness of that reimplementation,

including bugs and idiosyncrasies,

for the foreseeable future,

for _every_ database we intend to support. 

Even Sisyphus would pity us.

----

### Why does my project using sqlx query macros not build on docs.rs?

Docs.rs doesn't have access to your database, so it needs to be provided a `sqlx-data.json` file and be instructed to set the `SQLX_OFFLINE` environment variable to true while compiling your project. Luckily for us, docs.rs creates a `DOCS_RS` environment variable that we can access in a custom build script to achieve this functionality.

To do so, first, make sure that you have run `cargo sqlx prepare` to generate a `sqlx-data.json` file in your project.

Next, create a file called `build.rs` in the root of your project directory (at the same level as `Cargo.toml`). Add the following code to it:
```rs
fn main() {
    // When building in docs.rs, we want to set SQLX_OFFLINE mode to true
    if std::env::var_os("DOCS_RS").is_some() {
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    }
}
```
