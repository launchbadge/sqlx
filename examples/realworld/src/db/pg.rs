use async_trait::async_trait;
use sqlx::PgPool;

use super::model::*;
use anyhow::Error;

pub async fn connect(db_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::new(db_url).await?;
    Ok(pool)
}

#[async_trait]
impl ProvideUser for PgPool {
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> anyhow::Result<i32> {
        let rec = sqlx::query!(
            r#"
INSERT INTO users ( username, email, password )
VALUES ( $1, $2, $3 )
RETURNING id
        "#,
            username,
            email,
            password
        )
        .fetch_one(self)
        .await?;
        Ok(rec.id)
    }

    async fn get_user_by_id(&self, user_id: i32) -> anyhow::Result<UserEntity> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"
SELECT username, email, id, password
FROM users
WHERE id = $1
        "#,
            user_id
        )
        .fetch_one(self)
        .await?;

        Ok(rec)
    }

    async fn get_user_by_email(&self, email: &str) -> anyhow::Result<UserEntity> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"
SELECT username, email, id, password
FROM users
WHERE email = $1
            "#,
            email
        )
        .fetch_one(self)
        .await?;

        Ok(rec)
    }
}

#[async_trait]
impl ProvideArticle for PgPool {
    async fn create_article(&self) -> Result<ArticleEntity, Error> {
        unimplemented!()
    }

    async fn update_article(&self) -> Result<ArticleEntity, Error> {
        unimplemented!()
    }

    async fn delete_article(&self) -> Result<ArticleEntity, Error> {
        unimplemented!()
    }
}
