# Real World SQLx

## Usage

Declare the database URL.

```
export DATABASE_URL="postgres://postgres@localhost/realworld"
```

Create the database.

```
createdb -U postgres realworld
```

Load the database schema.

```
psql -d "$DATABASE_URL" -f ./schema.sql
``` 

Run.

```
cargo run
```
