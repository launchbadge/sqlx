# actix-sqlx-todo

Example Todo API using [Actix-web](https://github.com/actix/actix-web) and SQLx with posgresql

# Usage

## Prerequisites

* Rust
* PostgreSQL

## Change into the project sub-directory

All instructions assume you have changed into this folder:

```bash
cd examples/postgres/todo-api
```

## Set up the database

* Create new database using `schema.sql`
* Copy `.env-example` into `.env` and adjust DATABASE_URL to match your PostgreSQL address, username and password 

## Run the application

To run the application execute:

```bash
cargo run
```

By default application will be available on `http://localhost:5000`. If you wish to change address or port you can do it inside `.env` file.