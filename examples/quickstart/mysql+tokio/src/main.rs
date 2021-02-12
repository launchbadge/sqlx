use sqlx::mysql::{MySqlConnectOptions, MySqlConnection};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    // start by parsing the connection URL (typically from an environment variable)
    // you can also call [MySqlConnectOptions::new] to avoid any parsing

    let mut conn: MySqlConnection = MySqlConnectOptions::parse("mysql://root@localhost/Chinook")?
        // set a password (perhaps from somewhere else than the rest of the URL)
        .password("password")
        // connect to the database
        .connect()
        .await?;

    // ping, this makes sure the server is still there
    // hopefully it is - we did just connect to it

    conn.ping().await?;

    // make a simple query to list all
    // the rows from `MediaType`

    let rows = conn.fetch_all("SELECT * FROM MediaType").await?;

    for row in rows {
        let media_type_id: u8 = row.try_get(0)?;
        let media_type_name: &str = row.try_get(1)?;

        println!("media type, id: {}, name: {}", media_type_id, media_type_name);
    }

    // inherent query methods on the connection are unprepared queries and
    // offer no parameterization; however, you may submit multiple queries
    // in one, separated by `;`

    // for significantly better performance (and parameters), you will want
    // to use a prepared query

    // prepared queries are transparently cached and re-used (by hashing
    // the query string)

    let rows = sqlx::query("SELECT * FROM Track WHERE MediaTypeId = {id} LIMIT 5")
        .bind_as("id", &1)
        .fetch_all(&mut conn)
        .await?;

    for row in rows {
        // [...]
    }

    // to reduce boilerplate, a [FromRow] derive is available and can be
    // used with the [query_as] variant

    #[derive(sqlx::FromRow)]
    struct Genre {
        #[sqlx(rename = "GenreId")]
        id: u8,

        #[sqlx(rename = "Name")]
        name: String,
    }

    let genres: Vec<Genre> = sqlx::query_as("SELECT * FROM Genre").fetch_all(&mut conn).await?;

    for genre in genres {
        // [...]
    }

    // now time to get to the real magic 🪄 of SQLx
    // let's use the type-checked query macros

    let media_types = sqlx::query!("SELECT * FROM MediaType").fetch_all(&mut conn).await?;

    for media_type in media_types {
        println!("media type, id: {}, name: {}", media_type.media_type_id, media_type.name);
    }

    // yep, that's really it
    // as part of the type checker, an anonymous struct is produced

    // if you want to use a named struct, no derives are necessary, just match
    // the Rust type to the SQL type

    struct MediaType {
        // NOTE: when using the dynamic API, getting as u8 worked above
        //       however, the type-checked API demands strict typing to match Rust
        media_type_id: i32,
        name: String,
    }

    let target_id = 3;
    let media_type_for_3 =
        // NOTE: parameters are passed to the type-checked macro like println!
        sqlx::query_as!(MediaType, "SELECT * FROM MediaType WHERE MediaTypeId = {target_id}")
            // NOTE: fetch_one returns *one* row or record and will raise RowNotFound for no rows
            //       fetch_optional returns `None` instead for no rows
            .fetch_one(&mut conn)
            .await?;

    println!("media type, id: {}, name: {}", media_type_for_3.media_type_id, media_type_for_3.name);

    // close the connection explicitly
    // this kindly informs the database server that we'll be terminating
    // while not strictly required, the server will dispose of connection resources faster

    conn.close().await?;

    Ok(())
}
