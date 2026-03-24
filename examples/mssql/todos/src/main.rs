use clap::{Parser, Subcommand};
use sqlx::mssql::MssqlPool;
use sqlx::Row;
use std::env;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Add { description: String },
    Done { id: i64 },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let pool = MssqlPool::connect(&env::var("DATABASE_URL")?).await?;

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

async fn add_todo(pool: &MssqlPool, description: String) -> anyhow::Result<i64> {
    // MSSQL uses OUTPUT INSERTED instead of RETURNING
    let rec = sqlx::query("INSERT INTO todos (description) OUTPUT INSERTED.id VALUES (@p1)")
        .bind(&description)
        .fetch_one(pool)
        .await?;

    Ok(rec.get::<i64, _>("id"))
}

async fn complete_todo(pool: &MssqlPool, id: i64) -> anyhow::Result<bool> {
    let rows_affected = sqlx::query("UPDATE todos SET done = 1 WHERE id = @p1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

    Ok(rows_affected > 0)
}

async fn list_todos(pool: &MssqlPool) -> anyhow::Result<()> {
    let recs = sqlx::query("SELECT id, description, done FROM todos ORDER BY id")
        .fetch_all(pool)
        .await?;

    for rec in recs {
        println!(
            "- [{}] {}: {}",
            if rec.get::<bool, _>("done") { "x" } else { " " },
            rec.get::<i64, _>("id"),
            rec.get::<String, _>("description"),
        );
    }

    Ok(())
}
