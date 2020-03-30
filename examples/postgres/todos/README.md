# TODOs Example

## Usage

Declare the database URL:

```
export DATABASE_URL="postgres://postgres@localhost/todos"
```

Create the database:

```
createdb -U postgres todos
```

Load the database schema:

```
psql -d "$DATABASE_URL" -f ./schema.sql
```

Run:

- Add a todo: `cargo run -- add "todo description"`
- Complete a todo: `cargo run -- done <todo id>`
- List all todos: `cargo run`


