use anyhow::{Result, Error};
use async_trait::async_trait;
use sqlx::SqlitePool;

use super::model::*;

pub async fn connect(db_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::new(db_url).await?;
    Ok(pool)
}

#[async_trait]
impl ProvideUser for SqlitePool {
    async fn create_user(&self, username: &str, email: &str, password: &str) -> Result<i32> {
        use sqlx::sqlite::SqliteQueryAs;
        // Make a new transaction (for giggles)
        let mut tx = self.begin().await?;

        let rows_inserted = sqlx::query!(
            r#"INSERT INTO users ( username, email, password )
                VALUES ( $1, $2, $3 )"#,
            username,
            email,
            password
        )
        .execute(&mut tx)
        .await?;

        let (id,) = sqlx::query_as::<_, (i32,)>(r#"SELECT LAST_INSERT_ROWID()"#)
            .fetch_one(&mut tx)
            .await?;

        // FIXME(sgg): Potential bug, when I forget to commit the transaction
        //  the sqlite locked the table forever for some reason...
        // Explicitly commit (otherwise this would rollback on drop)
        tx.commit().await?;
        Ok(id)
    }

    async fn get_user_by_id(&self, user_id: i32) -> Result<UserEntity> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"SELECT id, email, username, password
                FROM users
                WHERE id = $1"#,
            user_id
        )
        .fetch_one(self)
        .await?;
        Ok(rec)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<UserEntity> {
        let rec = sqlx::query_as!(
            UserEntity,
            r#"SELECT id, email, username, password
            FROM users
            WHERE email = $1"#,
            email
        )
        .fetch_one(self)
        .await?;
        Ok(rec)
    }
}

#[async_trait]
impl ProvideArticle for SqlitePool {
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
