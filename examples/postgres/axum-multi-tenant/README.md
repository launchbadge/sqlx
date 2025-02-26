# Axum App with Multi-tenant Database

This example project involves three crates, each owning a different schema in one database,
with their own set of migrations.

* The main crate, an Axum app.
    * Owns the `public` schema (tables are referenced unqualified).
* `accounts`: a subcrate simulating a reusable account-management crate.
    * Owns schema `accounts`.
* `payments`: a subcrate simulating a wrapper for a payments API.
    * Owns schema `payments`.

## Note: Schema-Qualified Names

This example uses schema-qualified names everywhere for clarity.

It can be tempting to change the `search_path` of the connection (MySQL, Postgres) to eliminate the need for schema
prefixes, but this can cause some really confusing issues when names conflict.

This example will generate a `_sqlx_migrations` table in three different schemas, and if `search_path` is set
to `public,accounts,payments` and the migrator for the main application attempts to reference the table unqualified,
it would throw an error.
