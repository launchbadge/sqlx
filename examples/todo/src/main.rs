#![feature(async_await)]

use failure::Fallible;
use futures::{future, TryStreamExt};
use sqlx::postgres::Connection;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    cmd: Option<Command>
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "add")]
    Add {
        text: String,
    },

    #[structopt(name = "done")]
    MarkAsDone { id: i64 }
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let opt = Options::from_args();

    let mut conn = Connection::establish(
        sqlx::ConnectOptions::new()
            .host("127.0.0.1")
            .port(5432)
            .user("postgres")
            .database("sqlx__dev__tasks"),
    )
    .await?;

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

async fn ensure_schema(conn: &mut Connection) -> Fallible<()> {
    conn.prepare("BEGIN").execute().await?;

    // language=sql
    conn.prepare(
        r#"
CREATE TABLE IF NOT EXISTS tasks (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    done_at TIMESTAMPTZ
)
        "#,
    )
    .execute()
    .await?;

    conn.prepare("COMMIT").execute().await?;

    Ok(())
}

async fn print_all_tasks(conn: &mut Connection) -> Fallible<()> {
    // language=sql
    conn.prepare(
        r#"
SELECT id, text
FROM tasks
WHERE done_at IS NULL
        "#,
    )
    .select()
    .try_for_each(|(id, text): (i64, String)| {
        // language=text
        println!("{:>5} | {}", id, text);

        future::ok(())
    })
    .await?;

    Ok(())
}

async fn add_task(conn: &mut Connection, text: &str) -> Fallible<()> {
    // language=sql
    conn.prepare(
        r#"
INSERT INTO tasks ( text )
VALUES ( $1 )
        "#,
    )
        .bind(text)
        .execute()
        .await?;

    Ok(())
}

async fn mark_task_as_done(conn: &mut Connection, id: i64) -> Fallible<()> {
    // language=sql
    conn.prepare(
        r#"
UPDATE tasks
SET done_at = now()
WHERE id = $1
        "#,
    )
        .bind(id)
        .execute()
        .await?;

    Ok(())
}
