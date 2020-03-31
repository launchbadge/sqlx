# TODOs Example

## Usage

Declare the database URL:

```
export DATABASE_URL="mysql://localhost/todos"
```

Connect to `mysql` and create the database:

```
$ mysql
mysql> CREATE DATABASE todos;
```

Load the database schema (using the MySQL CLI interface thats already open):

```
mysql> USE todos;
mysql> source schema.sql
```

Use `exit` to exit the MySQL CLI. Then, to run this example:

- Add a todo: `cargo run -- add "todo description"`
- Complete a todo: `cargo run -- done <todo id>`
- List all todos: `cargo run`
