Postgres LISTEN/NOTIFY
======================

## Usage

Declare the database URL. This example does not include any reading or writing of data.

```
export DATABASE_URL="postgres://postgres@localhost/postgres"
```

Run.

```
cargo run
```

The example program should connect to the database, and create a LISTEN loop on a predefined set of channels. A NOTIFY task will be spawned which will connect to the same database and will emit notifications on a 5 second interval.
