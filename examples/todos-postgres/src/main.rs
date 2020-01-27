use sqlx::PgPool;
use std::env;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    Add { description: String },
    Done { id: i64 },
}

#[async_std::main]
#[paw::main]
async fn main(args: Args) -> anyhow::Result<()> {
    let mut pool = PgPool::new(&env::var("DATABASE_URL")?).await?;

    match args.cmd {
        Some(Command::Add { description }) => {
            println!("Adding new todo with description '{}'", &description);
            println!(
                "Added new todo with id {}",
                add_todo(&pool, &description).await.unwrap()
            );
        }
        Some(Command::Done { id }) => {
            println!("Marking todo {} as done", id);
            if complete_todo(&pool, id).await.unwrap() {
                println!("Todo {} is marked as done", id);
            } else {
                println!("Invalid id {}", id);
            }
        }
        None => {
            println!("Printing list of all todos");
            list_todos(&mut pool).await.unwrap();
        }
    }

    pool.close().await;

    Ok(())
}

async fn add_todo(pool: &PgPool, description: &str) -> Result<i64, sqlx::error::Error> {
    let mut tx = pool.begin().await?;

    let rec = sqlx::query!(
        r#"
INSERT INTO todos ( description )
VALUES ( $1 )
RETURNING id
        "#,
        description.to_string()
    )
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(rec.id)
}

async fn complete_todo(pool: &PgPool, id: i64) -> Result<bool, sqlx::error::Error> {
    let mut tx = pool.begin().await?;

    let rows_affected = sqlx::query!(
        r#"
UPDATE todos
SET done = TRUE
WHERE id = $1
        "#,
        id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(rows_affected > 0)
}

async fn list_todos(pool: &mut PgPool) -> Result<(), sqlx::error::Error> {
    let recs = sqlx::query!(
        r#"
SELECT id, description, done
FROM todos
ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!(
            "- [{}] {}: {}",
            match rec.done {
                true => "x",  // Done
                false => " ", // Not Done
            },
            rec.id,
            &rec.description,
        );
    }

    Ok(())
}
