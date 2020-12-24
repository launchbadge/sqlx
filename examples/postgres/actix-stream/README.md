# This uses a sqlx stream in an actix-web json API.

This example illustrates one way (among others) of using sqlx's
streaming capabilities.
  * 
This example illustrates a JSON REST API implemened by async streams.
A async stream is an async iterator. Upon calling stream.next().await,
the stream will respond with Ready(data) or Pending.

A query to http://localhost:8080/widigets will return a json array of
rows of widget details.  The /widgets API call is an actix-web service.
It builds sqlx query that returns a stream of rows.  The rows are
converted to json, and returned to actix-web as a stream 
of json text.

## Setup

Edit .env to set database URL and the loging level.

``` sh
RUST_LOG=info
DATABASE_URL=postgresql:///actixstream?host=/var/run/postgresql
```

Create the database.

    ``` sh
    # psql postgres -c 'create database actixstream;'
    ```

Setup the widgets table and import sample data

    ``` sh
    # psql actixstream -f migrations/setup_widgets.sql
    ```

## Usage

Run the web server.

``` sh
cargo run
```
And while the above is runing, issue a query.
``` sh
# curl -X POST -H 'Content-Type: application/json' -d '{"begin":0,"end":99999,"where_":null}' http://127.0.0.1:8080/widgets |jq
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   300    0   263  100    37  32875   4625 --:--:-- --:--:-- --:--:-- 33333
[
  {
    "id": 1,
    "serial": 10138,
    "name": "spanner",
    "description": "blue 10 guage joint spanner"
  },
  {
    "id": 2,
    "serial": 39822,
    "name": "flexarm",
    "description": "red flexible support arm"
  },
  {
    "id": 3,
    "serial": 52839,
    "name": "bearing",
    "description": "steel bearing for articulating joints"
  }
]
```

## Discussion

In this example, when a stream is passed to actix-web, it in turn
requires that the sql query text and database connection lifetimes
outlive the stream itself.  To do that, this example uses a
self-referntial struct implemented using the ouroboros crate.

This may seem verbose, but what it offers in return in this example
are: 1) low latency for the initial few rows, and 2) it allows one to
adjust the block size of data streamed to actix-web.  In principle,
adjusting the block size may be an advantage matching network, or
memory constraints, such as a 1500 byte block size for conventional
networks or 8192 if the network uses jumbo frames. 

This just illustrates just one way of workign with a stream of rows
that are converted and pass as a stream of bytes to actix-web.
