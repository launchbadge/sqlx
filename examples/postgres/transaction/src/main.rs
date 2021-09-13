use sqlx::query;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = sqlx::PgPool::connect(&conn_str).await?;

    let mut transaction = pool.begin().await?;

    let test_id = 1;
    query!(
        r#"INSERT INTO todos (id, description)
        VALUES ( $1, $2 )
        "#,
        test_id,
        "test todo"
    )
    .execute(&mut transaction)
    .await?;

    // check that inserted todo can be fetched
    let _ = query!(r#"SELECT FROM todos WHERE id = $1"#, test_id)
        .fetch_one(&mut transaction)
        .await?;

    transaction.rollback();

    // check that inserted todo is now gone
    let inserted_todo = query!(r#"SELECT FROM todos WHERE id = $1"#, test_id)
        .fetch_one(&pool)
        .await;

    assert!(inserted_todo.is_err());

    Ok(())
}
