/// Import as different namespace.
///
/// This must be set as `SQLX_NAMESPACE` environment variable to test that
/// changing the namespace still results in the proc macros behaving well.
extern crate sqlx as external;

#[test]
#[cfg(feature = "migrate")]
fn test_macros_namespace_migrate() {
    let _ = external::migrate!("../tests/migrate/migrations_simple");
}

#[test]
#[cfg(feature = "derive")]
fn test_macros_namespace_derive() {
    #[derive(Debug, external::Type, external::FromRow)]
    struct Test {
        value: i32,
    }
}
