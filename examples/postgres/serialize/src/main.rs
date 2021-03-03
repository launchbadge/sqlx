use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use std::io::{self, Read};
use std::num::NonZeroU8;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    Add,
}

#[derive(Deserialize, Serialize)]
struct Person {
    name: String,
    age: NonZeroU8,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

#[async_std::main]
#[paw::main]
async fn main(args: Args) -> anyhow::Result<()> {
    let pool = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    match args.cmd {
        Some(Command::Add) => {
            let mut json = String::new();
            io::stdin().read_to_string(&mut json)?;

            let person: Person = serde_json::from_str(&json)?;
            println!(
                "Adding new person: {}",
                &serde_json::to_string_pretty(&person)?
            );

            let person_id = add_person(&pool, person).await?;
            println!("Added new person with ID {}", person_id);
        }
        None => {
            println!("{}", list_people(&pool).await?);
        }
    }

    Ok(())
}

async fn add_person(pool: &PgPool, person: Person) -> anyhow::Result<i64> {
    let rec = sqlx::query!(
        r#"
INSERT INTO people ( person )
VALUES ( $1 )
RETURNING id
        "#,
        Json(person) as _
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

async fn list_people(pool: &PgPool) -> anyhow::Result<String> {
    let mut buf = String::from("[");
    for (i, row) in sqlx::query!(
        r#"
SELECT id, person
FROM people
ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?
    .iter()
    .enumerate()
    {
        if i > 0 {
            buf.push_str(",\n");
        }
        buf.push_str(&serde_json::to_string_pretty(&row)?);
    }
    buf.push_str("]\n");
    Ok(buf)
}
