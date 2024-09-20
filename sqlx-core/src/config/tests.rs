use crate::config::{self, Config};
use std::collections::BTreeSet;

#[test]
fn reference_parses_as_config() {
    let config: Config = toml::from_str(include_str!("reference.toml"))
        // The `Display` impl of `toml::Error` is *actually* more useful than `Debug`
        .unwrap_or_else(|e| panic!("expected reference.toml to parse as Config: {e}"));

    assert_common_config(&config.common);
    assert_macros_config(&config.macros);
    assert_migrate_config(&config.migrate);
}

fn assert_common_config(config: &config::common::Config) {
    assert_eq!(config.database_url_var.as_deref(), Some("FOO_DATABASE_URL"));
}

fn assert_macros_config(config: &config::macros::Config) {
    use config::macros::*;

    assert_eq!(config.preferred_crates.date_time, DateTimeCrate::Chrono);

    // Type overrides
    // Don't need to cover everything, just some important canaries.
    assert_eq!(config.type_override("foo"), Some("crate::types::Foo"));

    assert_eq!(config.type_override(r#""Bar""#), Some("crate::types::Bar"),);

    assert_eq!(
        config.type_override(r#""Foo".bar"#),
        Some("crate::schema::foo::Bar"),
    );

    assert_eq!(
        config.type_override(r#""Foo"."Bar""#),
        Some("crate::schema::foo::Bar"),
    );

    // Column overrides
    assert_eq!(
        config.column_override("foo", "bar"),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override("foo", r#""Bar""#),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override(r#""Foo""#, "bar"),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override(r#""Foo""#, r#""Bar""#),
        Some("crate::types::Bar"),
    );

    assert_eq!(
        config.column_override("my_schema.my_table", "my_column"),
        Some("crate::types::MyType"),
    );

    assert_eq!(
        config.column_override(r#""My Schema"."My Table""#, r#""My Column""#),
        Some("crate::types::MyType"),
    );
}

fn assert_migrate_config(config: &config::migrate::Config) {
    use config::migrate::*;

    assert_eq!(config.table_name.as_deref(), Some("foo._sqlx_migrations"));
    assert_eq!(config.migrations_dir.as_deref(), Some("foo/migrations"));

    let ignored_chars = BTreeSet::from([' ', '\t', '\r', '\n', '\u{FEFF}']);

    assert_eq!(config.ignored_chars, ignored_chars);

    assert_eq!(config.defaults.migration_type, DefaultMigrationType::Reversible);
    assert_eq!(config.defaults.migration_versioning, DefaultVersioning::Sequential);
}
