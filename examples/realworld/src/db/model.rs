use std::convert::TryFrom;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Error as SqlxError;

pub type EntityId = i32;

/// A user that is registered with the application
///
/// This entity is used to for authN/authZ
pub struct UserEntity {
    pub user_id: EntityId,
    pub email: String,
    pub username: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

/// A type that can provide stable storage for user authentication
///
/// This provider is used for managing users and passwords
#[async_trait]
pub trait ProvideAuthn {
    async fn create_user(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> ProvideResult<EntityId>;

    async fn get_user_by_id(&mut self, user_id: EntityId) -> ProvideResult<UserEntity>;

    async fn get_user_by_email(&mut self, email: &str) -> ProvideResult<UserEntity>;

    async fn update_user(&mut self, updated: &UserEntity) -> ProvideResult<()>;
}

/// A profile for an author of an article or comment
///
/// These should map 1:1 with users
#[derive(Default)]
pub struct ProfileEntity {
    pub user_id: EntityId,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

/// An artifact authored by a user
pub struct ArticleEntity {
    pub article_id: EntityId,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub body: String,
    pub author_id: EntityId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A comment on an article
pub struct CommentEntity {
    pub comment_id: EntityId,
    pub body: String,
    pub article_id: EntityId,
    pub author_id: EntityId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A type that provides access to stable storage for application data
#[async_trait]
pub trait ProvideData {
    async fn create_article(
        &mut self,
        author_id: EntityId,
        title: &str,
        slug: &str,
        description: &str,
        body: &str,
    ) -> ProvideResult<ArticleEntity>;

    async fn create_tags_for_article(
        &mut self,
        article_id: EntityId,
        tags: &'async_trait [impl AsRef<str> + Send + Sync],
    ) -> ProvideResult<()>;

    async fn update_article(&mut self, updated: &ArticleEntity) -> ProvideResult<ArticleEntity>;

    async fn delete_article(&mut self, slug: &str) -> ProvideResult<()>;

    async fn get_article_by_slug(&mut self, slug: &str) -> ProvideResult<ArticleEntity>;

    /// Retrieve all articles and authors
    async fn get_all_articles(&mut self) -> ProvideResult<Vec<(ArticleEntity, ProfileEntity)>>;

    async fn get_favorites_count(&mut self, article_slug: &str) -> ProvideResult<usize>;

    async fn create_favorite(&mut self, user_id: EntityId, article_slug: &str)
        -> ProvideResult<()>;

    async fn delete_favorite(&mut self, user_id: EntityId, article_slug: &str)
        -> ProvideResult<()>;

    async fn get_tags(&mut self) -> ProvideResult<Vec<String>>;

    async fn create_comment(
        &mut self,
        article_slug: &str,
        author_id: EntityId,
        body: &str,
    ) -> ProvideResult<CommentEntity>;

    async fn delete_comment(
        &mut self,
        article_slug: &str,
        comment_id: EntityId,
    ) -> ProvideResult<()>;

    async fn get_comment(
        &mut self,
        article_slug: &str,
        comment_id: EntityId,
    ) -> ProvideResult<CommentEntity>;

    async fn get_comments_on_article(
        &mut self,
        article_slug: &str,
    ) -> ProvideResult<Vec<(CommentEntity, ProfileEntity)>>;

    async fn get_profile_by_username(&mut self, username: &str) -> ProvideResult<ProfileEntity>;

    async fn get_profile_by_id(&mut self, profile_id: EntityId) -> ProvideResult<ProfileEntity>;

    async fn add_follower(
        &mut self,
        leader_username: &str,
        follower_id: EntityId,
    ) -> ProvideResult<()>;

    async fn delete_follower(
        &mut self,
        leader_username: &str,
        follower_id: EntityId,
    ) -> ProvideResult<()>;

    async fn is_following(
        &mut self,
        leader_id: EntityId,
        follower_id: EntityId,
    ) -> ProvideResult<bool>;

    /// Get users that are being followed by a user
    async fn get_following(&mut self, follower_id: EntityId) -> ProvideResult<Vec<EntityId>>;
}

pub type ProvideResult<T> = Result<T, ProvideError>;

/// An error returned by a provider
#[derive(Debug, thiserror::Error)]
pub enum ProvideError {
    /// The requested entity does not exist
    #[error("Entity does not exist")]
    NotFound,
    /// The operation violates a uniqueness constraint
    #[error("{0}")]
    UniqueViolation(String),
    /// The requested operation violates the data model
    #[error("{0}")]
    ModelViolation(String),
    #[error(transparent)]
    /// A generic unhandled error
    Provider(sqlx::Error),
}

impl From<SqlxError> for ProvideError {
    /// Convert a SQLx error into a provider error
    ///
    /// For Database errors we attempt to downcast
    ///
    /// FIXME(RFC): I have no idea if this is sane
    fn from(e: SqlxError) -> Self {
        log::debug!("sqlx returned err -- {:#?}", &e);
        match e {
            SqlxError::RowNotFound => ProvideError::NotFound,
            SqlxError::Database(db_err) => {
                #[cfg(feature = "postgres")]
                {
                    if let Some(pg_err) = db_err.try_downcast_ref::<sqlx::postgres::PgError>() {
                        if let Ok(provide_err) = ProvideError::try_from(pg_err) {
                            return provide_err;
                        }
                    }
                }

                #[cfg(feature = "sqlite")]
                {
                    if let Some(sqlite_err) = db_err.try_downcast_ref::<sqlx::sqlite::SqliteError>()
                    {
                        if let Ok(provide_err) = ProvideError::try_from(sqlite_err) {
                            return provide_err;
                        }
                    }
                }

                ProvideError::Provider(SqlxError::Database(db_err))
            }
            _ => ProvideError::Provider(e),
        }
    }
}
