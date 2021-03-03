# JSON Example using serialize feature

When the serialize feature is enabled, the query!() macro returns a
struct that implements serde::Serialize.  This means that each 'Row'
value can be converted to json text using serde_json::to_string(&row).
This includes nested 'jsonb', such as the person column in this
example.

## Setup

1. Declare the database URL

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/serialize"
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

Add a person

```
echo '{ "name": "John Doe", "age": 30 }' | cargo run -- add
```

or with extra keys

```
echo '{ "name": "Jane Doe", "age": 25, "array": ["string", true, 0] }' | cargo run -- add
```

List all people

```
cargo run
```
