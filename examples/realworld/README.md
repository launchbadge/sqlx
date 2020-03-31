# Real World SQLx

An implementation of ["The mother of all demo apps"](https://realworld.io/) using SQLx

This application supports both SQLite and PostgreSQL!

## Usage

1. Pick a DB Backend.

    ```
    export DB_TYPE="postgres"
    ```

2. Declare the database URL.

    ```
    export DATABASE_URL="postgres://postgres@localhost/realworld"
    ```

3. Create the database.

    ```
    createdb -U postgres realworld
    ```

4. Load the database schema from the appropriate file in [schema](./schema) directory.

    ```
    psql -d "${DATABASE_URL}" -f ./schema/postgres.sql
    ``` 

5. Run!

    ```
    cargo run --features "${DB_TYPE}" -- --db "${DB_TYPE}
    ```
   
6. Send some requests!

    ```
    curl --request POST \
      --url http://localhost:8080/api/users \
      --header 'content-type: application/json' \
      --data '{"user":{"email":"sqlx_user@foo.baz", "password":"not_secure", "username":"sqlx_user"}}'
    ```
   
    ```
    curl --request POST \
      --url http://localhost:8080/api/users/login \
      --header 'content-type: application/json' \
      --data '{"user":{"email":"sqlx_user@foo.baz", "password":"not_secure"}}'
    ```
