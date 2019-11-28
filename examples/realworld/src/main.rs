use sqlx::{FromRow, Pool, Postgres};
use std::env;
use tide::{http::StatusCode, Request, Response, ResultExt};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    let pool = Pool::<Postgres>::new(&env::var("DATABASE_URL")?).await?;

    let mut server = tide::with_state(pool);

    server.at("/api/users").post(register);

    server.listen(("localhost", 8080)).await?;

    Ok(())
}

// Registration
// https://github.com/gothinkster/realworld/tree/master/api#registration

// #[post("/api/users")]
async fn register(mut req: Request<Pool<Postgres>>) -> Response {
    #[derive(serde::Deserialize)]
    struct RegisterRequestBody {
        username: String,
        email: String,
        // TODO: password: String,
    }

    // TODO: Handle the unwrap
    let body: RegisterRequestBody = req.body_json().await.unwrap();
    let mut pool = req.state();

    // TODO: Handle the unwrap
    let (user_id,): (i64,) =
        sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id")
            .bind(body.username)
            .bind(body.email)
            .fetch_one(&mut pool)
            .await
            .unwrap();

    #[derive(serde::Serialize)]
    struct RegisterResponseBody {
        id: i64,
    }

    // TODO: Handle the unwrap
    Response::new(200)
        .body_json(&RegisterResponseBody { id: user_id })
        .unwrap()
}
