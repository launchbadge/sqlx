use sqlx_core::postgres::{PgConnection, Postgres};
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
async fn dynamic_ordering() -> anyhow::Result<()> {
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
async fn dynamic_filtering() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Article {
        id: i32,
        title: String,
        author: String,
    }

    enum Filter {
        Id(i32),
        Title(String),
        Author(String),
    }

    async fn query_articles(
        con: &mut PgConnection,
        filter: Option<Filter>,
    ) -> anyhow::Result<Vec<Article>> {
        let articles = sqlx::query_as!(
            Article,
            "SELECT"
                r#"id AS "id!", title AS "title!", author AS "author!""#
            "FROM ("
                "VALUES (1, 'Article1', 'Peter'), (2, 'Article2', 'John'), (3, 'Article3', 'James')"
            ") articles(id, title, author)"
            if let Some(filter) = filter {
                "WHERE"
                match filter {
                    Filter::Id(id) => "id = {id}",
                    Filter::Title(title) => "title ILIKE {title}",
                    Filter::Author(author) => "author ILIKE {author}"
                }
            }
        )
        .fetch_all(&mut conn)
        .await?;
        Ok(articles)
    }

    let result = query_articles(&mut conn, None).await?;
    assert_eq!(result.len(), 3);

    let result = query_articles(&mut conn, Some(Filter::Id(1))).await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 1);

    let result = query_articles(&mut conn, Some(Filter::Title("article2".to_owned()))).await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 2);

    let result = query_articles(&mut conn, Some(Filter::Author("james".to_owned()))).await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 3);

    Ok(())
}
