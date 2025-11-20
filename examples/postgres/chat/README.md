# Chat Example

Note: this example has an interactive TUI which is not trivial to test automatically,
so our CI currently only checks whether or not it compiles.

## Description

This example demonstrates how to use PostgreSQL channels to create a very simple chat application.

## Setup

1. Declare the database URL

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/files"
    ```

## Usage

Run the project

```
cargo run -p sqlx-examples-postgres-chat
```
