#![cfg_attr(
    not(any(feature = "postgres", feature = "mysql", feature = "offline")),
    allow(dead_code, unused_macros, unused_imports)
)]
#![cfg_attr(
    any(sqlx_macros_unstable, procmacro2_semver_exempt),
    feature(track_path, proc_macro_tracked_env)
)]
extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

type Error = Box<dyn std::error::Error>;

type Result<T> = std::result::Result<T, Error>;

mod common;
mod database;
mod derives;
mod query;

#[cfg(feature = "migrate")]
mod migrate;

#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

/// A variant of [query!] which takes a path to an explicitly defined struct as the output type.
///
/// This lets you return the struct from a function or add your own trait implementations.
///
/// **No trait implementations are required**; the macro maps rows using a struct literal
/// where the names of columns in the query are expected to be the same as the fields of the struct
/// (but the order does not need to be the same). The types of the columns are based on the
/// query and not the corresponding fields of the struct, so this is type-safe as well.
///
/// This enforces a few things:
/// * The query must output at least one column.
/// * The column names of the query must match the field names of the struct.
/// * The field types must be the Rust equivalent of their SQL counterparts; see the corresponding
/// module for your database for mappings:
///     * Postgres: [crate::postgres::types]
///     * MySQL: [crate::mysql::types]
///     * SQLite: [crate::sqlite::types]
///     * MSSQL: [crate::mssql::types]
/// * If a column may be `NULL`, the corresponding field's type must be wrapped in `Option<_>`.
/// * Neither the query nor the struct may have unused fields.
///
/// The only modification to the `query!()` syntax is that the struct name is given before the SQL
/// string:
/// ```rust,ignore
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "_rt-async-std"))]
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
/// # #[cfg(any(not(feature = "mysql"), not(feature = "_rt-async-std")))]
/// # fn main() {}
/// ```
///
/// **The method you want to call depends on how many rows you're expecting.**
///
/// | Number of Rows | Method to Call*             | Returns (`T` being the given struct)   | Notes |
/// |----------------| ----------------------------|----------------------------------------|-------|
/// | Zero or One    | `.fetch_optional(...).await`| `sqlx::Result<Option<T>>`              | Extra rows are ignored. |
/// | Exactly One    | `.fetch_one(...).await`     | `sqlx::Result<T>`                      | Errors if no rows were returned. Extra rows are ignored. Aggregate queries, use this. |
/// | At Least One   | `.fetch(...)`               | `impl Stream<Item = sqlx::Result<T>>`  | Call `.try_next().await` to get each row result. |
/// | Multiple       | `.fetch_all(...)`           | `sqlx::Result<Vec<T>>`  | |
///
/// \* All methods accept one of `&mut {connection type}`, `&mut Transaction` or `&Pool`.
/// (`.execute()` is omitted as this macro requires at least one column to be returned.)
///
/// ### Column Type Override: Infer from Struct Field
/// In addition to the column type overrides supported by [query!], `query_as!()` supports an
/// additional override option:
///
/// If you select a column `foo as "foo: _"` (Postgres/SQLite) or `` foo as `foo: _` `` (MySQL)
/// it causes that column to be inferred based on the type of the corresponding field in the given
/// record struct. Runtime type-checking is still done so an error will be emitted if the types
/// are not compatible.
///
/// This allows you to override the inferred type of a column to instead use a custom-defined type:
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[sqlx(transparent)]
/// struct MyInt4(i32);
///
/// struct Record {
///     id: MyInt4,
/// }
///
/// let my_int = MyInt4(1);
///
/// // Postgres/SQLite
/// sqlx::query_as!(Record, r#"select 1 as "id: _""#) // MySQL: use "select 1 as `id: _`" instead
///     .fetch_one(&mut conn)
///     .await?;
///
/// assert_eq!(record.id, MyInt4(1));
/// ```
///
/// ### Troubleshooting: "error: mismatched types"
/// If you get a "mismatched types" error from an invocation of this macro and the error
/// isn't pointing specifically at a parameter.
///
/// For example, code like this (using a Postgres database):
///
/// ```rust,ignore
/// struct Account {
///     id: i32,
///     name: Option<String>,
/// }
///
/// let account = sqlx::query_as!(
///     Account,
///     r#"SELECT id, name from (VALUES (1, 'Herp Derpinson')) accounts(id, name)"#,
/// )
///     .fetch_one(&mut conn)
///     .await?;
/// ```
///
/// Might produce an error like this:
/// ```text,ignore
/// error[E0308]: mismatched types
///    --> tests/postgres/macros.rs:126:19
///     |
/// 126 |       let account = sqlx::query_as!(
///     |  ___________________^
/// 127 | |         Account,
/// 128 | |         r#"SELECT id, name from (VALUES (1, 'Herp Derpinson')) accounts(id, name)"#,
/// 129 | |     )
///     | |_____^ expected `i32`, found enum `std::option::Option`
///     |
///     = note: expected type `i32`
///                found enum `std::option::Option<i32>`
/// ```
///
/// This means that you need to check that any field of the "expected" type (here, `i32`) matches
/// the Rust type mapping for its corresponding SQL column (see the `types` module of your database,
/// listed above, for mappings). The "found" type is the SQL->Rust mapping that the macro chose.
///
/// In the above example, the returned column is inferred to be nullable because it's being
/// returned from a `VALUES` statement in Postgres, so the macro inferred the field to be nullable
/// and so used `Option<i32>` instead of `i32`. **In this specific case** we could use
/// `select id as "id!"` to override the inferred nullability because we know in practice
/// that column will never be `NULL` and it will fix the error.
///
/// Nullability inference and type overrides are discussed in detail in the docs for [query!].
///
/// It unfortunately doesn't appear to be possible right now to make the error specifically mention
/// the field; this probably requires the `const-panic` feature (still unstable as of Rust 1.45).
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro]
pub fn query_as(input: TokenStream) -> TokenStream {
    match query::query_as(input.into()) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Encode, attributes(sqlx))]
pub fn derive_encode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_encode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derives::expand_derive_from_row(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "migrate")]
#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    use syn::LitStr;

    let input = syn::parse_macro_input!(input as LitStr);
    match migrate::expand_migrator_from_dir(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    let result = if cfg!(feature = "_rt-tokio") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                ::sqlx_rt::tokio::runtime::Builder::new_multi_thread()
                    .enable_io()
                    .enable_time()
                    .build()
                    .unwrap()
                    .block_on(async { #body })
            }
        }
    } else if cfg!(feature = "_rt-async-std") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                ::sqlx_rt::async_std::task::block_on(async { #body })
            }
        }
    } else if cfg!(feature = "_rt-actix") {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                ::sqlx_rt::actix_rt::System::new()
                    .block_on(async { #body })
            }
        }
    } else {
        panic!("one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled");
    };

    result.into()
}
