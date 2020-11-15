# SQLx CLI

SQLx's associated command-line utility for managing databases, migrations, and enabling "offline"
mode with `sqlx::query!()` and friends.

### Install

#### With Rust toolchain

```bash
# supports all databases supported by SQLx
$ cargo install --version=0.2.0 sqlx-cli

# only for postgres
$ cargo install --version=0.2.0 sqlx-cli --no-default-features --features postgres
```

### Usage

All commands require that a database url is provided. This can be done either with the `--database-url` command line option or by setting `DATABASE_URL`, either in the environment or in a `.env` file
in the current working directory.

`database` and `migrate` subcommands support only Postgres; MySQL and SQLite are TODO.

For more details, run `sqlx <command> --help`.

```dotenv
# Postgres
DATABASE_URL=postgres://postgres@localhost/my_database
```

#### Create/drop the database at `DATABASE_URL`

```bash
sqlx database create
sqlx database drop
```

#### Create and run migrations

```bash
$ sqlx migrate add <name>
```
Creates a new file in `migrations/<timestamp>-<name>.sql`. Add your database schema changes to
this new file.

---
```bash
$ sqlx migrate run
```
Compares the migration history of the running database against the `migrations/` folder and runs
any scripts that are still pending.

#### Enable building in "offline" mode with `query!()`

Note: must be run as `cargo sqlx`.

```bash
cargo sqlx prepare
```
Saves query data to `sqlx-data.json` in the current directory; check this file into version control
and an active database connection will no longer be needed to build your project.

Has no effect unless the `offline` feature of `sqlx` is enabled in your project. Omitting that feature is the most likely cause if you get a `sqlx-data.json` file that looks like this:

```json
{
    "database": "PostgreSQL"
}
```

----
```bash
cargo sqlx prepare --check
```
Exits with a nonzero exit status if the data in `sqlx-data.json` is out of date with the current
database schema and queries in the project. Intended for use in Continuous Integration.

#### Force building in offline mode

To make sure an accidentally-present `DATABASE_URL` environment variable or `.env` file does not
result in `cargo build` (trying to) access the database, you can set the `SQLX_OFFLINE` environment
variable to `true`.
