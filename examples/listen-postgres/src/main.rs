use std::time::Duration;

use async_std::stream;
use futures::stream::StreamExt;
use sqlx::postgres::PgPoolExt;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building PG pool.");
    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = sqlx::PgPool::new(&conn_str).await?;

    let notify_pool = pool.clone();
    let _t = async_std::task::spawn(async move {
        stream::interval(Duration::from_secs(5))
            .for_each(move |_| notify(notify_pool.clone()))
            .await
    });

    println!("Starting LISTEN loop.");
    let mut listener = pool.listen(&["chan0", "chan1", "chan2"]);
    let mut counter = 0usize;
    loop {
        let res = listener.recv().await;
        println!("[from recv]: {:?}", res);
        counter += 1;
        if counter >= 3 {
            break;
        }
    }

    let stream = listener.into_stream();
    futures::pin_mut!(stream);
    while let Some(res) = stream.next().await {
        println!("[from stream]: {:?}", res);
    }

    Ok(())
}

async fn notify(pool: sqlx::PgPool) {
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(err) => return println!("[from notify]: {:?}", err),
    };
    let res = sqlx::Executor::send(
        &mut conn,
        r#"
        NOTIFY "chan0", '{"payload": 0}';
        NOTIFY "chan1", '{"payload": 1}';
        NOTIFY "chan2", '{"payload": 2}';
    "#,
    )
    .await;
    println!("[from notify]: {:?}", res);
}
