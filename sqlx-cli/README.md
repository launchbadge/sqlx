# SQLx CLI

SQLx's associated command-line utility for managing databases, migrations, and enabling "offline"
mode with `sqlx::query!()` and friends.

### Installation

```bash
$ cargo install sqlx-cli
```

### Commands

All commands require `DATABASE_URL` to be set, either in the environment or in a `.env` file
in the current working directory.

`database` and `migrate` subcommands support only Postgres; MySQL and SQLite are TODO.

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
$ sqlx migration run
```
Compares the migration history of the running database against the `migrations/` folder and runs
any scripts that are still pending.

##### Note: Down-Migrations
Down-migrations are currently a non-planned feature as their utility seems dubious but we welcome 
any contributions (discussions/code) regarding this matter.

#### Enable building in "offline" mode with `query!()` 
Note: must be run as `cargo sqlx`.

```bash
cargo sqlx prepare
```
Saves query data to `sqlx-data.json` in the current directory; check this file into version control
and an active database connection will no longer be needed to build your project.
----
```bash
cargo sqlx prepare --check
```
Exits with a nonzero exit status if the data in `sqlx-data.json` is out of date with the current
database schema and queries in the project. Intended for use in Continuous Integration.
