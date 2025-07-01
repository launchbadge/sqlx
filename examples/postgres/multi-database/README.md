# Using Multiple Databases with `sqlx.toml`

This example project involves three crates, each owning a different schema in one database,
with their own set of migrations.

* The main crate, a simple binary simulating the action of a REST API.
    * Owns the `public` schema (tables are referenced unqualified).
    * Migrations are moved to `src/migrations` using config key `migrate.migrations-dir`
      to visually separate them from the subcrate folders.
* `accounts`: a subcrate simulating a reusable account-management crate.
    * Owns schema `accounts`.
* `payments`: a subcrate simulating a wrapper for a payments API.
    * Owns schema `payments`.

## Note: Schema-Qualified Names

This example uses schema-qualified names everywhere for clarity.

It can be tempting to change the `search_path` of the connection (MySQL, Postgres) to eliminate the need for schema
prefixes, but this can cause some really confusing issues when names conflict.

This example will generate a `_sqlx_migrations` table in three different schemas; if `search_path` is set
to `public,accounts,payments` and the migrator for the main application attempts to reference the table unqualified,
it would throw an error.

# Setup

This example requires running three different sets of migrations.

Ensure `sqlx-cli` is installed with Postgres and `sqlx.toml` support:

```
cargo install sqlx-cli --features postgres,sqlx-toml
```

Start a Postgres server (shown here using Docker, `run` command also works with `podman`):

```
docker run -d -e POSTGRES_PASSWORD=password -p 5432:5432 --name postgres postgres:latest
```

Create `.env` with the various database URLs or set them in your shell environment;

```
DATABASE_URL=postgres://postgres:password@localhost/example-multi-database
ACCOUNTS_DATABASE_URL=postgres://postgres:password@localhost/example-multi-database-accounts
PAYMENTS_DATABASE_URL=postgres://postgres:password@localhost/example-multi-database-payments
```

Run the following commands:

```
(cd accounts && sqlx db setup)
(cd payments && sqlx db setup)
sqlx db setup
```

It is an open question how to make this more convenient; `sqlx-cli` could gain a `--recursive` flag that checks
subdirectories for `sqlx.toml` files, but that would only work for crates within the same workspace. If the `accounts`
and `payments` crates were instead crates.io dependencies, we would need Cargo's help to resolve that information.

An issue has been opened for discussion: <https://github.com/launchbadge/sqlx/issues/3761>
