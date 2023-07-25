use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::{env, io::Write, sync::Arc};
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::from_args_safe()?;
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let todo_repo = PostgresTodoRepo::new(pool);
    let mut writer = std::io::stdout();

    handle_command(args, todo_repo, &mut writer).await
}

async fn handle_command(
    args: Args,
    todo_repo: impl TodoRepo,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    match args.cmd {
        Some(Command::Add { description }) => {
            writeln!(
                writer,
                "Adding new todo with description '{}'",
                &description
            )?;
            let todo_id = todo_repo.add_todo(description).await?;
            writeln!(writer, "Added new todo with id {todo_id}")?;
        }
        Some(Command::Done { id }) => {
            writeln!(writer, "Marking todo {id} as done")?;
            if todo_repo.complete_todo(id).await? {
                writeln!(writer, "Todo {id} is marked as done")?;
            } else {
                writeln!(writer, "Invalid id {id}")?;
            }
        }
        None => {
            writeln!(writer, "Printing list of all todos")?;
            todo_repo.list_todos().await?;
        }
    }

    Ok(())
}

#[mockall::automock]
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[async_std::test]
    async fn test_mocked_add() {
        let description = String::from("My todo");
        let args = Args {
            cmd: Some(Command::Add {
                description: description.clone(),
            }),
        };

        let mut todo_repo = MockTodoRepo::new();
        todo_repo
            .expect_add_todo()
            .times(1)
            .with(eq(description))
            .returning(|_| Ok(1));

        let mut writer = Vec::new();

        handle_command(args, todo_repo, &mut writer).await.unwrap();

        assert_eq!(
            String::from_utf8_lossy(&writer),
            "Adding new todo with description \'My todo\'\nAdded new todo with id 1\n"
        );
    }
}
