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


## Unimplemented Features

* Filters via query parameters
* Unit tests

## Known Issues/Quirks

* This is not a production application, pks are public ids, caveat emptor, etc.
* Currently you CANNOT compile this crate with multiple DB backends enabled as the query macros
    will conflict with one another.
* SQLite locks the tables if there are basically any errors (e.g. constraint violations). This may be related to
    [#193](https://github.com/launchbadge/sqlx/issues/193)
* The realworld API tests complain about timestamps in our responses.
    This is an issue w/ their tests [gothinkster/realworld#490]https://github.com/gothinkster/realworld/pull/490
* As of `0.6.0`, `tide` has not fully worked out the error handling story.
    `tide::ResultExt` helps but as of now API endpoint functions can only return `tide::Response`
* `sqlx::Error` does not carry type information about the Database so some clever downcasting
    is needed to resolve details from Database errors
