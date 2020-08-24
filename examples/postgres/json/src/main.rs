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

struct Row {
    id: i64,
    person: Json<Person>,
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
            println!("Printing all people");
            list_people(&pool).await?;
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

async fn list_people(pool: &PgPool) -> anyhow::Result<()> {
    let rows = sqlx::query_as!(
        Row,
        r#"
SELECT id, person as "person: Json<Person>"
FROM people
ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        println!(
            "{}: {}",
            row.id,
            &serde_json::to_string_pretty(&row.person)?
        );
    }

    Ok(())
}
