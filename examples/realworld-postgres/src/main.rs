use sqlx::PgPool;
use std::env;
use tide::{Request, Response};

// NOTE: Tide 0.5.x does not handle errors so any fallible methods just [.unwrap] for the moment.
//       To be clear, that is not recommended and this should be fixed as soon as Tide fixes its
//       error handling.

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    let pool = PgPool::new(&env::var("DATABASE_URL")?).await?;

    let mut server = tide::with_state(pool);

    server.at("/api/users").post(register);

    server.listen(("0.0.0.0", 8080)).await?;

    Ok(())
}

// Registration
// https://github.com/gothinkster/realworld/tree/master/api#registration

// #[post("/api/users")]
async fn register(mut req: Request<PgPool>) -> Response {
    #[derive(serde::Deserialize)]
    struct RegisterRequestBody {
        username: String,
        email: String,
        password: String,
    }

    let body: RegisterRequestBody = req.body_json().await.unwrap();

    let mut pool = req.state();

    let (user_id,): (i64,) = sqlx::query!(
        r#"
INSERT INTO users ( username, email )
VALUES ( $1, $2 )
RETURNING id
            "#,
        &*body.username,
        &*body.email
    )
    .fetch_one(&mut pool)
    .await
    .unwrap();

    #[derive(serde::Serialize)]
    struct RegisterResponseBody {
        id: i64,
    }

    Response::new(200)
        .body_json(&RegisterResponseBody { id: user_id })
        .unwrap()
}
