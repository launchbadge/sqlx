# TODOs Example

## Usage

Declare the database URL:

```
export DATABASE_URL="sqlite:///path/to/this/directory/todos.db"
```

Create the database:

```
sqlite3 todos.db
```

Load the database schema (using the SQLite CLI interface opened from the previous command):

```
sqlite> .read schema.sql
```

Use `.exit` to leave the SQLite CLI. Then, to run this example:

- Add a todo: `cargo run -- add "todo description"`
- Complete a todo: `cargo run -- done <todo id>`
- List all todos: `cargo run`
