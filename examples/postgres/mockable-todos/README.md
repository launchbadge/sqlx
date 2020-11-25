# Mockable TODOs Example

## Description

This example is based on the ideas in [this blog post](https://medium.com/better-programming/structuring-rust-project-for-testability-18207b5d0243). The value here is that the business logic can be unit tested independently from the database layer. Otherwise it is identical to the todos example.

## Setup

1. Run `docker-compose up -d` to run Postgres in the background.

2. Declare the database URL, either by exporting it:

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/todos"
    ```

    or by making a `.env` file:

    ```
    cp .env.example .env
    ```

3. Create the database.

    ```
    $ sqlx db create
    ```

4. Run sql migrations

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

## Cleanup

To destroy the Postgres database, run:

```
docker-compose down --volumes
```
