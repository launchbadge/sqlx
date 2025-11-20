# Query files Example

## Description

This example demonstrates storing external files to use for querying data.
Encapsulating your SQL queries can be helpful in several ways, assisting with intellisense,
etc.


## Setup

1. Declare the database URL

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/files"
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

Run the project

```
cargo run files
```