# JSON Example

## Setup

1. Declare the database URL

    ```
    export DATABASE_URL="postgres://postgres:password@localhost/json"
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
