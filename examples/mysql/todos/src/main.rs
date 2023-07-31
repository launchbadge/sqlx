use sqlx::mysql::MySqlPool;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args_safe()?;
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;

    match args.cmd {
        Some(Command::Add { description }) => {
            println!("Adding new todo with description '{description}'");
            let todo_id = add_todo(&pool, description).await?;
            println!("Added new todo with id {todo_id}");
        }
        Some(Command::Done { id }) => {
            println!("Marking todo {id} as done");
            if complete_todo(&pool, id).await? {
                println!("Todo {id} is marked as done");
            } else {
                println!("Invalid id {id}");
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
    // Insert the task, then obtain the ID of this row
    let todo_id = sqlx::query!(
        r#"
INSERT INTO todos ( description )
VALUES ( ? )
        "#,
        description
    )
    .execute(pool)
    .await?
    .last_insert_id();

    Ok(todo_id)
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
    .await?
    .rows_affected();

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

    // NOTE: Booleans in MySQL are stored as `TINYINT(1)` / `i8`
    //       0 = false, non-0 = true
    for rec in recs {
        println!(
            "- [{}] {}: {}",
            if rec.done != 0 { "x" } else { " " },
            rec.id,
            &rec.description,
        );
    }

    Ok(())
}
