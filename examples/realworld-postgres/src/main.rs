use chrono::{Duration, Utc};
use rand::{thread_rng, RngCore};
use sqlx::PgPool;
use std::env;
use tide::{Request, Response};

const SECRET_KEY: &str = "this-is-the-most-secret-key-ever-secreted";

// NOTE: Tide 0.5.x does not handle errors so any fallible methods just [.unwrap] for the moment.
//       To be clear, that is not recommended and this should be fixed as soon as Tide fixes its
//       error handling.

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPool::new(&env::var("DATABASE_URL")?).await?;

    let mut server = tide::with_state(pool);

    server.at("/api/users").post(register);

    server.at("/api/user").get(get_current_user);

    server.listen(("localhost", 8080)).await?;

    Ok(())
}

// User
// https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication

#[derive(serde::Serialize)]
struct User {
    email: String,
    token: String,
    username: String,
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
    let hash = hash_password(&body.password).unwrap();

    // Make a new transaction (for giggles)
    let pool = req.state();
    let mut tx = pool.begin().await.unwrap();

    let rec = sqlx::query!(
        r#"
INSERT INTO users ( username, email, password )
VALUES ( $1, $2, $3 )
RETURNING id, username, email
        "#,
        body.username,
        body.email,
        hash
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    let token = generate_token(rec.id).unwrap();

    // Explicitly commit (otherwise this would rollback on drop)
    tx.commit().await.unwrap();

    #[derive(serde::Serialize)]
    struct RegisterResponseBody {
        user: User,
    }

    Response::new(200)
        .body_json(&RegisterResponseBody {
            user: User {
                username: rec.username,
                email: rec.email,
                token,
            },
        })
        .unwrap()
}

// Get Current User
// https://github.com/gothinkster/realworld/tree/master/api#get-current-user

// #[get("/api/user")]
async fn get_current_user(req: Request<PgPool>) -> Response {
    // TODO: Combine these methods? &Request isn't Sync though
    let token = get_token_from_request(&req);
    let user_id = authorize(&token).await.unwrap();

    let mut pool = req.state();

    let rec = sqlx::query!(
        r#"
SELECT username, email
FROM users
WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&mut pool)
    .await
    .unwrap();

    #[derive(serde::Serialize)]
    struct GetCurrentUserResponseBody {
        user: User,
    }

    Response::new(200)
        .body_json(&GetCurrentUserResponseBody {
            user: User {
                username: rec.username,
                email: rec.email,
                token,
            },
        })
        .unwrap()
}

fn get_token_from_request(req: &Request<PgPool>) -> String {
    req.header("authorization")
        .unwrap_or_default()
        .splitn(2, ' ')
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

async fn authorize(token: &str) -> anyhow::Result<i64> {
    let data = jsonwebtoken::decode::<TokenClaims>(
        token,
        SECRET_KEY.as_ref(),
        &jsonwebtoken::Validation::default(),
    )?;

    Ok(data.claims.sub)
}

// TODO: Does this need to be spawned in async-std ?
fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = generate_random_salt();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

    Ok(hash)
}

fn generate_random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    thread_rng().fill_bytes(&mut salt);

    salt
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TokenClaims {
    sub: i64,
    exp: i64,
}

fn generate_token(user_id: i64) -> anyhow::Result<String> {
    use jsonwebtoken::Header;

    let exp = Utc::now() + Duration::hours(1);
    let token = jsonwebtoken::encode(
        &Header::default(),
        &TokenClaims {
            sub: user_id,
            exp: exp.timestamp(),
        },
        SECRET_KEY.as_ref(),
    )?;

    Ok(token)
}
