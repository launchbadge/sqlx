/// Statically checked SQL query with `println!()` style syntax.
///
/// This expands to an instance of [QueryAs] that outputs an ad-hoc anonymous struct type,
/// if the query has output columns, or `()` (unit) otherwise:
///
/// ```rust
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "runtime-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query!("select (1) as id, 'Herp Derpinson' as name")
///     .fetch_one(&mut conn)
///     .await?;
///
/// // anonymous struct has `#[derive(Debug)]` for convenience
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "runtime-async-std")))]
/// # fn main() {}
/// ```
///
/// ## Requirements
/// * The `DATABASE_URL` environment variable must be set at build-time to point to a database
/// server with the schema that the query string will be checked against. All variants of `query!()`
/// use [dotenv] so this can be in a `.env` file instead.
///
///     * Or, `sqlx-data.json` must exist at the workspace root. See [Offline Mode](#offline-mode)
///       below.
///
/// * The query must be a string literal or else it cannot be introspected (and thus cannot
/// be dynamic or the result of another macro).
///
/// * The `QueryAs` instance will be bound to the same database type as `query!()` was compiled
/// against (e.g. you cannot build against a Postgres database and then run the query against
/// a MySQL database).
///
///     * The schema of the database URL (e.g. `postgres://` or `mysql://`) will be used to
///       determine the database type.
///
/// [dotenv]: https://crates.io/crates/dotenv
/// ## Query Arguments
/// Like `println!()` and the other formatting macros, you can add bind parameters to your SQL
/// and this macro will typecheck passed arguments and error on missing ones:
///
/// ```rust
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "runtime-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::mysql::MySqlConnection::connect(db_url).await?;
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query!(
///         // just pretend "accounts" is a real table
///         "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
///         1i32
///     )
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "runtime-async-std")))]
/// # fn main() {}
/// ```
///
/// Bind parameters in the SQL string are specific to the database backend:
///
/// * Postgres: `$N` where `N` is the 1-based positional argument index
/// * MySQL: `?` which matches arguments in order that it appears in the query
///
/// ## Nullability: Bind Parameters
/// For a given expected type `T`, both `T` and `Option<T>` are allowed (as well as either
/// behind references). `Option::None` will be bound as `NULL`, so if binding a type behind `Option`
/// be sure your query can support it.
///
/// Note, however, if binding in a `where` clause, that equality comparisons with `NULL` may not
/// work as expected; instead you must use `IS NOT NULL` or `IS NULL` to check if a column is not
/// null or is null, respectively. Note that `IS [NOT] NULL` cannot be bound as a parameter either;
/// you must modify your query string instead.
///
/// ## Nullability: Output Columns
/// In most cases, the database engine can tell us whether or not a column may be `NULL`, and
/// the `query!()` macro adjusts the field types of the returned struct accordingly.
///
/// For Postgres and SQLite, this only works for columns which come directly from actual tables,
/// as the implementation will need to query the table metadata to find if a given column
/// has a `NOT NULL` constraint. Columns that do not have a `NOT NULL` constraint or are the result
/// of an expression are assumed to be nullable and so `Option<T>` is used instead of `T`.
///
/// For MySQL, the implementation looks at [the `NOT_NULL` flag](https://dev.mysql.com/doc/dev/mysql-server/8.0.12/group__group__cs__column__definition__flags.html#ga50377f5ca5b3e92f3931a81fe7b44043)
/// of [the `ColumnDefinition` structure in `COM_QUERY_OK`](https://dev.mysql.com/doc/internals/en/com-query-response.html#column-definition):
/// if it is set, `T` is used; if it is not set, `Option<T>` is used.
///
/// MySQL appears to be capable of determining the nullability of a result column even if it
/// is the result of an expression, depending on if the expression may in any case result in
/// `NULL` which then depends on the semantics of what functions are used. Consult the MySQL
/// manual for the functions you are using to find the cases in which they return `NULL`.
///
/// To override the nullability of an output column, use [query_as!].
///
/// ### Offline Mode (requires the `offline` feature)
/// The macros can be configured to not require a live database connection for compilation,
/// but it requires a couple extra steps:
///
/// * Run `cargo install sqlx-cli`.
/// * In your project with `DATABASE_URL` set (or in a `.env` file) and the database server running,
///   run `cargo sqlx prepare`.
/// * Check the generated `sqlx-data.json` file into version control.
/// * Don't have `DATABASE_URL` set during compilation.
///
/// Your project can now be built without a database connection (you must omit `DATABASE_URL` or
/// else it will still try to connect). To update the generated file simply run `cargo sqlx prepare`
/// again.
///
/// To ensure that your `sqlx-data.json` file is kept up-to-date, both with the queries in your
/// project and your database schema itself, run
/// `cargo install sqlx-cli && cargo sqlx prepare --check` in your Continuous Integration script.
///
/// See [the README for `sqlx-cli`](https://crates.io/crate/sqlx-cli) for more information.
///
/// ## See Also
/// * [query_as!] if you want to use a struct you can name,
/// * [query_file!] if you want to define the SQL query out-of-line,
/// * [query_file_as!] if you want both of the above.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query (
    // by emitting a macro definition from our proc-macro containing the result tokens,
    // we no longer have a need for `proc-macro-hack`
    ($query:literal) => ({
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source = $query);
        }
        macro_result!()
    });
    ($query:literal, $($args:expr),*$(,)?) => ({
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source = $query, args = [$($args),*]);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query!] which does not check the input or output types. This still does parse
/// the query to ensure it's syntactically and semantically valid for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_unchecked (
    ($query:literal) => ({
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source = $query, checked = false);
        }
        macro_result!()
    });
    ($query:literal, $($args:expr),*$(,)?) => ({
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source = $query, args = [$($args),*], checked = false);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query!] where the SQL query is stored in a separate file.
///
/// Useful for large queries and potentially cleaner than multiline strings.
///
/// The syntax and requirements (see [query!]) are the same except the SQL string is replaced by a
/// file path.
///
/// The file must be relative to the project root (the directory containing `Cargo.toml`),
/// unlike `include_str!()` which uses compiler internals to get the path of the file where it
/// was invoked.
///
/// -----
///
/// `examples/queries/account-by-id.sql`:
/// ```text
/// select * from (select (1) as id, 'Herp Derpinson' as name) accounts
/// where id = ?
/// ```
///
/// `src/my_query.rs`:
/// ```rust
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "runtime-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// let account = sqlx::query_file!("tests/test-query-account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "runtime-async-std")))]
/// # fn main() {}
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file (
    ($path:literal) => (#[allow(dead_code)]{
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source_file = $path);
        }
        macro_result!()
    });
    ($path:literal, $($args:expr),*$(,)?) => (#[allow(dead_code)]{
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(source_file = $path, args = [$($args),*]);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query_file!] which does not check the input or output types. This still does parse
/// the query to ensure it's syntactically and semantically valid for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_unchecked (
    ($path:literal) => (#[allow(dead_code)]{
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::query_file_unchecked!(source_file = $path, checked = false);
        }
        macro_result!()
    });
    ($path:literal, $($args:expr),*$(,)?) => (#[allow(dead_code)]{
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::query_file_unchecked!(source_file = $path, args = [$($args),*], checked = false);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query!] which takes a path to an explicitly defined struct as the output type.
///
/// This lets you return the struct from a function or add your own trait implementations.
///
/// No trait implementations are required; the macro maps rows using a struct literal
/// where the names of columns in the query are expected to be the same as the fields of the struct
/// (but the order does not need to be the same). The types of the columns are based on the
/// query and not the corresponding fields of the struct, so this is type-safe as well.
///
/// This enforces a few things:
/// * The query must output at least one column.
/// * The column names of the query must match the field names of the struct.
/// * Neither the query nor the struct may have unused fields.
///
/// The only modification to the syntax is that the struct name is given before the SQL string:
/// ```rust
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "runtime-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query_as!(
///         Account,
///         "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
///         1i32
///     )
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "runtime-async-std")))]
/// # fn main() {}
/// ```
///
/// ## Nullability
/// Use `Option` for columns which may be `NULL` in order to avoid a runtime error being returned
/// from `.fetch_*()`.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as (
    ($out_struct:path, $query:literal) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source = $query);
        }
        macro_result!()
    });
    ($out_struct:path, $query:literal, $($args:expr),*$(,)?) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source = $query, args = [$($args),*]);
        }
        macro_result!($($args),*)
    })
);

/// Combines the syntaxes of [query_as!] and [query_file!].
///
/// Enforces requirements of both macros; see them for details.
///
/// ```rust
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "runtime-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query_file_as!(Account, "tests/test-query-account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "runtime-async-std")))]
/// # fn main() {}
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_as (
    ($out_struct:path, $path:literal) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source_file = $path);
        }
        macro_result!()
    });
    ($out_struct:path, $path:literal, $($args:tt),*$(,)?) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args),*]);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query_as!] which does not check the input or output types. This still does parse
/// the query to ensure it's syntactically and semantically valid for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as_unchecked (
    ($out_struct:path, $query:literal) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source = $query, checked = false);
        }
        macro_result!()
    });

    ($out_struct:path, $query:literal, $($args:expr),*$(,)?) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::expand_query!(record = $out_struct, source = $query, args = [$($args),*], checked = false);
        }
        macro_result!($($args),*)
    })
);

/// A variant of [query_file_as!] which does not check the input or output types. This
/// still does parse the query to ensure it's syntactically and semantically valid
/// for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_as_unchecked (
    ($out_struct:path, $path:literal) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::query_file_as_unchecked!(record = $out_struct, source_file = $path, checked = false);
        }
        macro_result!()
    });

    ($out_struct:path, $path:literal, $($args:tt),*$(,)?) => (#[allow(dead_code)] {
        #[macro_use]
        mod _macro_result {
            $crate::sqlx_macros::query_file_as_unchecked!(record = $out_struct, source_file = $path, args = [$($args),*], checked = false);
        }
        macro_result!($($args),*)
    })
);
