#![feature(async_await)]

use failure::Fallible;
use futures::{future, TryStreamExt};
use sqlx::{Connection, Postgres};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "add")]
    Add { text: String },

    #[structopt(name = "done")]
    MarkAsDone { id: i64 },
}

#[tokio::main]
async fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let opt = Options::from_args();

    let mut conn =
        Connection::<Postgres>::establish("postgres://postgres@127.0.0.1/sqlx__dev").await?;

    ensure_schema(&mut conn).await?;

    match opt.cmd {
        Some(Command::Add { text }) => {
            add_task(&mut conn, &text).await?;
        }

        Some(Command::MarkAsDone { id }) => {
            mark_task_as_done(&mut conn, id).await?;
        }

        None => {
            print_all_tasks(&mut conn).await?;
        }
    }

    Ok(())
}

async fn ensure_schema(conn: &mut Connection<Postgres>) -> Fallible<()> {
    conn.execute("BEGIN", ()).await?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS tasks (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    done_at TIMESTAMPTZ
)
        "#,
    )
    .execute(conn)
    .await?;

    conn.execute("COMMIT", ()).await?;

    Ok(())
}

async fn print_all_tasks(conn: &mut Connection<Postgres>) -> Fallible<()> {
    conn.fetch(
        r#"
SELECT id, text
FROM tasks
WHERE done_at IS NULL
        "#,
    )
    .try_for_each(|(id, text): (i64, String)| {
        // language=text
        println!("{:>5} | {}", id, text);

        future::ok(())
    })
    .await?;

    Ok(())
}

async fn add_task(conn: &mut Connection<Postgres>, text: &str) -> Fallible<()> {
    conn.execute("INSERT INTO tasks ( text ) VALUES ( $1 )", (text,)).await?;

    Ok(())
}

async fn mark_task_as_done(conn: &mut Connection<Postgres>, id: i64) -> Fallible<()> {
    // language=sql
    sqlx::query(
        r#"
UPDATE tasks
SET done_at = now()
WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(conn)
    .await?;

    Ok(())
}
