use sqlx::mysql::MySqlQueryAs;
use sqlx::MySqlPool;
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
    Done { id: u64 },
}

#[async_std::main]
#[paw::main]
async fn main(args: Args) -> anyhow::Result<()> {
    let pool = MySqlPool::new(&env::var("DATABASE_URL")?).await?;

    match args.cmd {
        Some(Command::Add { description }) => {
            println!("Adding new todo with description '{}'", &description);
            let todo_id = add_todo(&pool, description).await?;
            println!("Added new todo with id {}", todo_id);
        }
        Some(Command::Done { id }) => {
            println!("Marking todo {} as done", id);
            if complete_todo(&pool, id).await? {
                println!("Todo {} is marked as done", id);
            } else {
                println!("Invalid id {}", id);
            }
        }
        None => {
            println!("Printing list of all todos");
            list_todos(&pool).await?;
        }
    }

    Ok(())
}

async fn add_todo(pool: &MySqlPool, description: String) -> anyhow::Result<u64> {
    // Insert the TODO, then obtain the ID of this row
    sqlx::query!(
        r#"
INSERT INTO todos ( description )
VALUES ( ? )
        "#,
        description
    )
    .execute(pool)
    .await?;

    let rec: (u64,) = sqlx::query_as("SELECT LAST_INSERT_ID()")
        .fetch_one(pool)
        .await?;

    Ok(rec.0)
}

async fn complete_todo(pool: &MySqlPool, id: u64) -> anyhow::Result<bool> {
    let rows_affected = sqlx::query!(
        r#"
UPDATE todos
SET done = TRUE
WHERE id = ?
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(rows_affected > 0)
}

async fn list_todos(pool: &MySqlPool) -> anyhow::Result<()> {
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
            if rec.done > 0 { "x" } else { " " },
            rec.id,
            &rec.description,
        );
    }

    Ok(())
}
