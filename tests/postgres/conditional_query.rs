use sqlx_core::postgres::Postgres;
use sqlx_test::new;

#[sqlx_macros::test]
async fn simple() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    struct Result {
        result: i32,
    }

    for value in [true, false] {
        let result = sqlx::query_as!(
            Result,
            "SELECT"
            if value { "42" } else { "12" }
            r#"AS "result""#
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
async fn single_if() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    #[derive(Clone, Eq, PartialEq. Debug)]
    struct Article {
        id: i32,
        title: String,
        author: String,
    }

    let expected = vec![
        Article {
            id: 1,
            title: "Article1".to_owned(),
            author: "Peter".to_owned(),
        },
        Article {
            id: 2,
            title: "Article2".to_owned(),
            author: "John".to_owned(),
        },
    ];
    for reverse_order in [true, false] {
        let articles = sqlx::query_as!(
            Article,
            "SELECT *"
            r#"FROM (VALUES (1, "Article1", "Peter"), (2, "Article2", "John"))"#
            "ORDER BY name"
            if reverse_order {
                "REV"
            }
        )
        .fetch_all(&mut conn)
        .await?;

        if reverse_order {
            let mut expected = expected.clone();
            expected.reverse();
            assert_eq!(articles, expected);
        } else {
            assert_eq!(articles, expected);
        }
    }

    for value in [true, false] {
        let result = sqlx::query_as!(
            Result,
            "SELECT"
            if value { "42" } else { "12" }
            r#"AS "result""#
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
