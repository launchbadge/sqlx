use async_trait::async_trait;
use sqlx::postgres::PgPool;
use sqlx::Done;
use std::{env, sync::Arc};
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
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let todo_repo = PostgresTodoRepo::new(pool);

    handle_command(args, todo_repo).await
}

async fn handle_command(args: Args, todo_repo: impl TodoRepo) -> anyhow::Result<()> {
    match args.cmd {
        Some(Command::Add { description }) => {
            println!("Adding new todo with description '{}'", &description);
            let todo_id = todo_repo.add_todo(description).await?;
            println!("Added new todo with id {}", todo_id);
        }
        Some(Command::Done { id }) => {
            println!("Marking todo {} as done", id);
            if todo_repo.complete_todo(id).await? {
                println!("Todo {} is marked as done", id);
            } else {
                println!("Invalid id {}", id);
            }
        }
        None => {
            println!("Printing list of all todos");
            todo_repo.list_todos().await?;
        }
    }

    Ok(())
}

#[async_trait]
pub trait TodoRepo {
    async fn add_todo(&self, description: String) -> anyhow::Result<i64>;
    async fn complete_todo(&self, id: i64) -> anyhow::Result<bool>;
    async fn list_todos(&self) -> anyhow::Result<()>;
}

struct PostgresTodoRepo {
    pg_pool: Arc<PgPool>,
}

impl PostgresTodoRepo {
    fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

#[async_trait]
impl TodoRepo for PostgresTodoRepo {
    async fn add_todo(&self, description: String) -> anyhow::Result<i64> {
        let rec = sqlx::query!(
            r#"
INSERT INTO todos ( description )
VALUES ( $1 )
RETURNING id
        "#,
            description
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(rec.id)
    }

    async fn complete_todo(&self, id: i64) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
UPDATE todos
SET done = TRUE
WHERE id = $1
        "#,
            id
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn list_todos(&self) -> anyhow::Result<()> {
        let recs = sqlx::query!(
            r#"
SELECT id, description, done
FROM todos
ORDER BY id
        "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        for rec in recs {
            println!(
                "- [{}] {}: {}",
                if rec.done { "x" } else { " " },
                rec.id,
                &rec.description,
            );
        }

        Ok(())
    }
}
