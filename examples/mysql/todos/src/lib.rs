use clap::{Parser, Subcommand};
use sqlx::mysql::MySqlPool;
use std::env;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Add { description: String },
    Done { id: u64 },
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse_from(wasip3::cli::environment::get_arguments());
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;

    match args.cmd {
        Some(Command::Add { description }) => {
            eprintln!("Adding new todo with description '{description}'");
            let todo_id = add_todo(&pool, description).await?;
            eprintln!("Added new todo with id {todo_id}");
        }
        Some(Command::Done { id }) => {
            eprintln!("Marking todo {id} as done");
            if complete_todo(&pool, id).await? {
                eprintln!("Todo {id} is marked as done");
            } else {
                eprintln!("Invalid id {id}");
            }
        }
        None => {
            eprintln!("Printing list of all todos");
            list_todos(&pool).await?;
        }
    }

    Ok(())
}

async fn add_todo(pool: &MySqlPool, description: String) -> anyhow::Result<u64> {
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

    for rec in recs {
        eprintln!(
            "- [{}] {}: {}",
            if rec.done != 0 { "x" } else { " " },
            rec.id,
            &rec.description,
        );
    }

    Ok(())
}

wasip3::cli::command::export!(Component);

struct Component;

impl wasip3::exports::cli::run::Guest for Component {
    async fn run() -> Result<(), ()> {
        if let Err(err) = run().await {
            let (mut tx, rx) = wasip3::wit_stream::new();

            futures::join!(
                async { wasip3::cli::stderr::write_via_stream(rx).await.unwrap() },
                async {
                    let remaining = tx.write_all(format!("{err:#}\n").into_bytes()).await;
                    assert!(remaining.is_empty());
                    drop(tx);
                }
            );
            Err(())
        } else {
            Ok(())
        }
    }
}
