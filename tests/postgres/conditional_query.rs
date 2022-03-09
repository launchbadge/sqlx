use sqlx_core::postgres::Postgres;
use sqlx_test::new;

#[sqlx_macros::test]
async fn simple() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    struct Result {
        result: i32
    }

    for value in [true, false] {
        let result = sqlx::query_as!(
            Result,
            "SELECT"
            if value { "42" } else { "12" }
        )
        .fetch_one(&mut conn)
        .await?;

        if value {
            assert_eq!(result.result, 42);
        } else {
            assert_eq!(result.result, 12);
        }
    }

    Ok(())
}

#[sqlx_macros::test]
async fn fail() -> anyhow::Result<()> {
    panic!("test this gets executed");
}
