use sqlx_core::postgres::Postgres;
use sqlx_test::new;

#[sqlx_macros::test]
async fn simple() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    struct Result {
        result: Option<i32>,
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
            assert_eq!(result.result, Some(42));
        } else {
            assert_eq!(result.result, Some(12));
        }
    }

    Ok(())
}

#[sqlx_macros::test]
async fn single_if() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    #[derive(Clone, Eq, PartialEq, Debug)]
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
            "SELECT"
                r#"id AS "id!", title AS "title!", author AS "author!""#
            "FROM ("
                "VALUES (1, 'Article1', 'Peter'), (2, 'Article2', 'John')"
            ") articles(id, title, author)"
            "ORDER BY title"
            if reverse_order {
                "DESC"
            } else {
                "ASC"
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

    Ok(())
}

#[sqlx_macros::test]
async fn fail() -> anyhow::Result<()> {
    panic!("test this gets executed");
}
