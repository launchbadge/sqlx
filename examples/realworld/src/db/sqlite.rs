use std::convert::TryFrom;

use anyhow::{Error, Result};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::error::DatabaseError;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteError, SqliteQueryAs};
use sqlx::Error as SqlxError;
use sqlx::{Connection, Cursor, Executor, FromRow, SqliteConnection, SqlitePool};

use crate::db::model::*;
use crate::db::Db;

impl TryFrom<&SqliteError> for ProvideError {
    type Error = ();

    /// Attempt to convert a Sqlite into a more-specific provider error
    ///
    /// Unexpected cases will be bounced back to the caller for handling
    ///
    /// * [Sqlite Error Codes](https://www.sqlite.org/rescode.html)
    fn try_from(db_err: &SqliteError) -> Result<Self, Self::Error> {
        let provider_err = match db_err.code().unwrap() {
            "2067" => ProvideError::UniqueViolation(db_err.message().to_owned()),
            _ => return Err(()),
        };

        Ok(provider_err)
    }
}

#[derive(sqlx::FromRow)]
struct SqliteArticleEntity {
    article_id: EntityId,
    title: String,
    slug: String,
    description: String,
    body: String,
    author_id: EntityId,
    created_at: i32,
    updated_at: i32,
}

impl From<SqliteArticleEntity> for ArticleEntity {
    fn from(entity: SqliteArticleEntity) -> Self {
        let SqliteArticleEntity {
            article_id,
            title,
            slug,
            description,
            body,
            author_id,
            created_at,
            updated_at,
        } = entity;

        ArticleEntity {
            article_id,
            title,
            slug,
            description,
            body,
            author_id,
            created_at: Utc.timestamp(created_at as _, 0),
            updated_at: Utc.timestamp(updated_at as _, 0),
        }
    }
}

#[derive(sqlx::FromRow)]
struct SqliteCommentEntity {
    comment_id: EntityId,
    body: String,
    article_id: EntityId,
    author_id: EntityId,
    created_at: EntityId,
    updated_at: EntityId,
}

impl From<SqliteCommentEntity> for CommentEntity {
    fn from(entity: SqliteCommentEntity) -> Self {
        let SqliteCommentEntity {
            comment_id,
            body,
            article_id,
            author_id,
            created_at,
            updated_at,
        } = entity;
        CommentEntity {
            comment_id,
            body,
            article_id,
            author_id,
            created_at: Utc.timestamp(created_at as _, 0),
            updated_at: Utc.timestamp(updated_at as _, 0),
        }
    }
}

pub async fn connect(db_url: &str) -> sqlx::Result<SqlitePool> {
    let pool = SqlitePool::new(db_url).await?;
    Ok(pool)
}

#[async_trait]
impl Db for SqlitePool {
    type Conn = PoolConnection<SqliteConnection>;

    async fn conn(&self) -> sqlx::Result<Self::Conn> {
        self.acquire().await
    }
}

#[async_trait]
impl ProvideAuthn for SqliteConnection {
    async fn create_user(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> ProvideResult<EntityId> {
        let (user_id,): (EntityId,) = sqlx::query_as(
            r#"
INSERT INTO users ( username, email, password )
VALUES ( $1, $2, $3 );
SELECT last_insert_rowid();
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password)
        .fetch_one(self)
        .await?;
        Ok(user_id)
    }

    async fn get_user_by_id(&mut self, user_id: EntityId) -> ProvideResult<UserEntity> {
        let user = sqlx::query_as!(
            UserEntity,
            r#"
SELECT user_id, username, email, password, image, bio
FROM users
WHERE user_id = $1
        "#,
            user_id
        )
        .fetch_one(self)
        .await?;
        Ok(user)
    }

    async fn get_user_by_email(&mut self, email: &str) -> ProvideResult<UserEntity> {
        let user = sqlx::query_as!(
            UserEntity,
            r#"
SELECT user_id, username, email, password, image, bio
FROM users
WHERE email = $1
        "#,
            email
        )
        .fetch_one(self)
        .await?;
        Ok(user)
    }

    async fn update_user(&mut self, updated: &UserEntity) -> ProvideResult<()> {
        sqlx::query!(
            r#"
UPDATE users
SET email = $1, username = $2, password = $3, image = $4, bio = $5, updated_at = (STRFTIME('%s', 'now'))
WHERE user_id = $6
            "#,
            updated.email,
            updated.username,
            updated.password,
            updated.image,
            updated.bio,
            updated.user_id,
        )
            .execute(self)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ProvideData for SqliteConnection {
    async fn create_article(
        &mut self,
        author_id: EntityId,
        title: &str,
        slug: &str,
        description: &str,
        body: &str,
    ) -> ProvideResult<ArticleEntity> {
        let rec: SqliteArticleEntity = sqlx::query_as(
            r#"
INSERT INTO articles ( title, slug, description, body, author_id )
VALUES ( $1, $2, $3, $4, $5);
SELECT * FROM articles WHERE article_id = last_insert_rowid();
            "#,
        )
        .bind(title)
        .bind(slug)
        .bind(description)
        .bind(body)
        .bind(author_id)
        .fetch_one(self)
        .await?;

        Ok(rec.into())
    }

    async fn create_tags_for_article(
        &mut self,
        article_id: EntityId,
        tags: &'async_trait [impl AsRef<str> + Send + Sync],
    ) -> ProvideResult<()> {
        let stmt = format!(
            r#"
INSERT INTO TAGS (tag_name, article_id)
VALUES {}
            "#,
            super::build_batch_insert(tags.len(), 2)
        );
        tags.iter()
            .fold(sqlx::query(&stmt), |q, tag_name| {
                q.bind(tag_name.as_ref()).bind(article_id)
            })
            .execute(self)
            .await?;

        Ok(())
    }

    async fn update_article(&mut self, updated: &ArticleEntity) -> ProvideResult<ArticleEntity> {
        self.execute("SAVEPOINT update_article").await?;

        let update_stmt = sqlx::query!(
            r#"
UPDATE articles
SET title = $2, slug = $3, description = $4, body = $5, updated_at = (STRFTIME('%s', 'now'))
WHERE article_id = $1
            "#,
            updated.article_id,
            updated.title,
            updated.slug,
            updated.description,
            updated.body,
        );

        self.execute(update_stmt).await?;

        let select_stmt =
            sqlx::query(r#"SELECT * FROM articles WHERE article_id = $1"#).bind(updated.article_id);

        let rec = self
            .fetch(select_stmt)
            .next()
            .await?
            .map(|row| SqliteArticleEntity::from_row(&row).expect("invalid entity"))
            .expect("Cursor should not be empty");

        self.execute("RELEASE update_article").await?;

        Ok(rec.into())
    }

    async fn delete_article(&mut self, slug: &str) -> Result<(), ProvideError> {
        sqlx::query!(r#"DELETE FROM articles WHERE slug = $1"#, slug)
            .execute(self)
            .await?;
        Ok(())
    }

    async fn get_article_by_slug(&mut self, slug: &str) -> Result<ArticleEntity, ProvideError> {
        let rec: SqliteArticleEntity = sqlx::query_as(
            r#"
SELECT *
FROM articles
WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_one(self)
        .await?;

        Ok(rec.into())
    }

    async fn get_all_articles(
        &mut self,
    ) -> Result<Vec<(ArticleEntity, ProfileEntity)>, ProvideError> {
        let recs = sqlx::query!(
            r#"
SELECT
    articles.*
    ,profiles.username, profiles.bio as bio, profiles.image
FROM articles
INNER JOIN profiles ON articles.author_id = profiles.user_id
ORDER BY created_at
            "#
        )
        .fetch_all(self)
        .await?;

        let entities = recs
            .into_iter()
            .map(|rec| {
                let article = SqliteArticleEntity {
                    article_id: rec.article_id,
                    title: rec.title,
                    slug: rec.slug,
                    description: rec.description,
                    body: rec.body,
                    author_id: rec.author_id,
                    created_at: rec.created_at,
                    updated_at: rec.updated_at,
                };
                let author = ProfileEntity {
                    user_id: rec.author_id,
                    username: rec.username,
                    bio: rec.bio,
                    image: rec.image,
                };
                (ArticleEntity::from(article), author)
            })
            .collect::<Vec<_>>();

        Ok(entities)
    }

    async fn get_favorites_count(&mut self, article_slug: &str) -> Result<usize, ProvideError> {
        // let (user_id, ): (EntityId, ) = sqlx::query_as(
        let (count,): (i64,) = sqlx::query_as(
            r#"
SELECT COUNT(favs.user_id) as count
FROM favorite_articles AS favs
INNER JOIN articles ON articles.article_id = favs.article_id
WHERE articles.slug = $1
            "#,
        )
        .bind(article_slug)
        .fetch_one(self)
        .await?;

        Ok(count as _)
    }

    async fn create_favorite(
        &mut self,
        user_id: EntityId,
        article_slug: &str,
    ) -> ProvideResult<()> {
        sqlx::query!(
            r#"
INSERT INTO favorite_articles ( user_id, article_id )
VALUES (
    $1
    ,( SELECT article_id FROM articles WHERE slug = $2 )
)
ON CONFLICT DO NOTHING
            "#,
            user_id,
            article_slug,
        )
        .execute(self)
        .await?;

        Ok(())
    }

    async fn delete_favorite(
        &mut self,
        user_id: EntityId,
        article_slug: &str,
    ) -> ProvideResult<()> {
        sqlx::query!(
            r#"
DELETE FROM favorite_articles
WHERE
    user_id = $1
    AND article_id = ( SELECT article_id FROM articles WHERE slug = $2 )
            "#,
            user_id,
            article_slug,
        )
        .execute(self)
        .await?;

        Ok(())
    }

    async fn get_tags(&mut self) -> ProvideResult<Vec<String>> {
        let tags = sqlx::query!(r#"SELECT DISTINCT tag_name from tags"#)
            .fetch_all(self)
            .await?;

        Ok(tags.into_iter().map(|rec| rec.tag_name).collect::<Vec<_>>())
    }

    async fn create_comment(
        &mut self,
        article_slug: &str,
        author_id: EntityId,
        body: &str,
    ) -> ProvideResult<CommentEntity> {
        self.execute("SAVEPOINT create_comment;").await?;

        let insert_stmt = sqlx::query!(
            r#"
INSERT INTO comments ( article_id, author_id , body )
VALUES (
    ( SELECT article_id FROM articles WHERE slug = $1 )
    , $2
    , $3
);
            "#,
            article_slug,
            author_id,
            body
        );

        self.execute(insert_stmt).await?;

        let rec = self
            .fetch("SELECT * FROM comments WHERE comment_id = last_insert_rowid()")
            .next()
            .await?
            .map(|row| SqliteCommentEntity::from_row(&row).expect("Invalid entity"))
            .expect("No row matching last_insert_rowid()");

        self.execute("RELEASE create_comment;").await?;

        Ok(rec.into())
    }

    async fn delete_comment(
        &mut self,
        article_slug: &str,
        comment_id: EntityId,
    ) -> ProvideResult<()> {
        sqlx::query!(
            r#"
DELETE FROM comments
WHERE
    article_id = ( SELECT article_id FROM articles WHERE slug = $1 )
    AND comment_id = $2
            "#,
            article_slug,
            comment_id,
        )
        .execute(self)
        .await?;

        Ok(())
    }

    async fn get_comment(
        &mut self,
        article_slug: &str,
        comment_id: EntityId,
    ) -> ProvideResult<CommentEntity> {
        let rec = sqlx::query_as!(
            SqliteCommentEntity,
            r#"
SELECT comments.*
FROM comments
INNER JOIN articles ON articles.slug = $1
WHERE comment_id = $2
            "#,
            article_slug,
            comment_id,
        )
        .fetch_one(self)
        .await?;

        Ok(rec.into())
    }

    async fn get_comments_on_article(
        &mut self,
        article_slug: &str,
    ) -> ProvideResult<Vec<(CommentEntity, ProfileEntity)>> {
        let recs = sqlx::query!(
            r#"
SELECT
    comments.*
    , profiles.username, profiles.bio, profiles.image
FROM comments
INNER JOIN articles ON articles.slug = $1
INNER JOIN profiles ON profiles.user_id = comments.author_id
            "#,
            article_slug
        )
        .fetch_all(self)
        .await?;

        let entities = recs
            .into_iter()
            .map(|rec| {
                let comment = SqliteCommentEntity {
                    comment_id: rec.comment_id,
                    body: rec.body,
                    article_id: rec.article_id,
                    author_id: rec.author_id,
                    created_at: rec.created_at,
                    updated_at: rec.updated_at,
                };
                let profile = ProfileEntity {
                    user_id: rec.author_id,
                    username: rec.username,
                    bio: rec.bio,
                    image: rec.image,
                };

                (CommentEntity::from(comment), profile)
            })
            .collect::<Vec<_>>();

        Ok(entities)
    }

    async fn get_profile_by_username(&mut self, username: &str) -> ProvideResult<ProfileEntity> {
        let rec = sqlx::query_as!(
            ProfileEntity,
            r#"
    SELECT user_id, username, bio, image
    FROM profiles
    WHERE username = $1
            "#,
            username,
        )
        .fetch_one(self)
        .await?;

        Ok(rec)
    }

    async fn get_profile_by_id(&mut self, profile_id: EntityId) -> ProvideResult<ProfileEntity> {
        let rec = sqlx::query_as!(
            ProfileEntity,
            r#"
SELECT user_id, username, bio, image
FROM profiles
WHERE user_id = $1
            "#,
            profile_id
        )
        .fetch_one(self)
        .await?;

        Ok(rec)
    }

    async fn add_follower(
        &mut self,
        leader_username: &str,
        follower_id: EntityId,
    ) -> ProvideResult<()> {
        sqlx::query!(
            r#"
INSERT INTO followers ( follower_id, leader_id )
VALUES (
    $1,
    ( SELECT user_id FROM users WHERE username = $2 )
)
ON CONFLICT DO NOTHING
        "#,
            follower_id,
            leader_username
        )
        .execute(self)
        .await?;

        Ok(())
    }

    async fn delete_follower(
        &mut self,
        leader_username: &str,
        follower_id: EntityId,
    ) -> ProvideResult<()> {
        sqlx::query!(
            r#"
DELETE FROM followers
WHERE
    leader_id = ( SELECT user_id FROM users WHERE username = $1 )
    AND follower_id = $2
            "#,
            leader_username,
            follower_id
        )
        .execute(self)
        .await?;

        Ok(())
    }

    async fn is_following(
        &mut self,
        leader_id: EntityId,
        follower_id: EntityId,
    ) -> ProvideResult<bool> {
        let rec = sqlx::query!(
            r#"
SELECT leader_id
FROM followers
WHERE leader_id = $1 AND follower_id = $2
        "#,
            leader_id,
            follower_id,
        )
        .fetch_optional(self)
        .await?;

        Ok(rec.is_some())
    }

    async fn get_following(&mut self, follower_id: EntityId) -> ProvideResult<Vec<EntityId>> {
        let recs = sqlx::query!(
            r#"
SELECT leader_id from followers
WHERE follower_id = $1
            "#,
            follower_id
        )
        .fetch_all(self)
        .await?;

        Ok(recs.into_iter().map(|rec| rec.leader_id).collect())
    }
}
