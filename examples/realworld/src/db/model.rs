use async_trait::async_trait;

pub struct UserEntity {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: Option<String>, // FIXME(RFC): Why is this nullable in the DB?
}

/// A type that can provide [`UserEntities`]
#[async_trait]
pub trait ProvideUser {
    async fn create_user(&self, username: &str, email: &str, password: &str)
        -> anyhow::Result<i32>;

    async fn get_user_by_id(&self, user_id: i32) -> anyhow::Result<UserEntity>;

    async fn get_user_by_email(&self, email: &str) -> anyhow::Result<UserEntity>;
}

pub struct ArticleEntity {
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
}

#[async_trait]
pub trait ProvideArticle {
    async fn create_article(&self) -> anyhow::Result<ArticleEntity>;

    async fn update_article(&self) -> anyhow::Result<ArticleEntity>;

    async fn delete_article(&self) -> anyhow::Result<ArticleEntity>;
}
