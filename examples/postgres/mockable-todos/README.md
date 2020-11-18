# Mockable TODOs Example

## Description

This example is based on the ideas in [this blog post](https://medium.com/better-programming/structuring-rust-project-for-testability-18207b5d0243). The value here is that the business logic can be unit tested independently from the database layer. Otherwise it is identical to the todos example.

## Setup

1. Declare the database URL

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/todos"
    ```

2. Create the database.

    ```
    $ sqlx db create
    ```

3. Run sql migrations

    ```
    $ sqlx migrate run
    ```

## Usage

Add a todo

```
cargo run -- add "todo description"
```

Complete a todo.

```
cargo run -- done <todo id>
```

List all todos

```
cargo run
```
