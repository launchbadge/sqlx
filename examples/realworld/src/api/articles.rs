use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use heck::KebabCase;
use log::*;
use serde::{Deserialize, Serialize};
use sqlx::pool::PoolConnection;
use sqlx::{Connect, Connection};
use tide::{Error, IntoResponse, Request, Response, ResultExt};

use crate::api::model::*;
use crate::api::util::*;
use crate::db::model::{ArticleEntity, CommentEntity, EntityId, ProfileEntity, ProvideData};
use crate::db::Db;
use std::collections::HashSet;
use std::iter::FromIterator;

/// The response body for a single article
///
/// [API Spec](https://github.com/gothinkster/realworld/tree/master/api#single-article)
#[derive(Serialize)]
struct ArticleResponseBody {
    article: Article,
}

/// The response body for multiple articles
///
/// [API Spec](https://github.com/gothinkster/realworld/tree/master/api#multiple-comments)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MultArticlesResponseBody {
    articles: Vec<Article>,
    articles_count: usize,
}

impl From<Vec<Article>> for MultArticlesResponseBody {
    fn from(articles: Vec<Article>) -> Self {
        let articles_count = articles.len();
        Self {
            articles,
            articles_count,
        }
    }
}

/// A comment on an article
///
/// [API Spec](https://github.com/gothinkster/realworld/tree/master/api#single-comment)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Comment {
    id: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    body: String,
    author: Profile,
}

impl Comment {
    /// Create a comment from DB entities with author.following populated based on the leaders
    fn with_leaders(
        entities: (CommentEntity, ProfileEntity),
        leader_ids: &HashSet<EntityId>,
    ) -> Self {
        let is_following = leader_ids.contains(&entities.0.author_id);
        let mut comment = Comment::from(entities);
        comment.author.following = is_following;

        comment
    }
}

impl From<(CommentEntity, Profile)> for Comment {
    fn from(data: (CommentEntity, Profile)) -> Self {
        let CommentEntity {
            comment_id,
            body,
            created_at,
            updated_at,
            ..
        } = data.0;

        let author = data.1;

        Comment {
            id: comment_id as _,
            created_at,
            updated_at,
            body,
            author,
        }
    }
}

impl From<(CommentEntity, ProfileEntity)> for Comment {
    fn from(entities: (CommentEntity, ProfileEntity)) -> Self {
        Comment::from((entities.0, Profile::from(entities.1)))
    }
}

#[derive(Serialize)]
struct CommentResponseBody {
    comment: Comment,
}

#[derive(Serialize)]
struct MultipleCommentsResponseBody {
    comments: Vec<Comment>,
}

/// Retrieve all articles
///
/// [List Articles](https://github.com/gothinkster/realworld/tree/master/api#list-articles)
pub async fn list_articles(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let state = req.state();

        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let authenticated = optionally_auth(&req).transpose()?;

        let entities = tx.get_all_articles().await?;

        let leader_ids: HashSet<EntityId> = if let Some((user_id, _)) = authenticated {
            HashSet::from_iter(tx.get_following(user_id).await?)
        } else {
            HashSet::default()
        };

        let articles = entities
            .into_iter()
            .map(|ents| Article::with_following(ents, &leader_ids))
            .collect::<Vec<_>>();

        tx.commit().await.server_err()?;

        let resp = Response::new(200)
            .body_json(&MultArticlesResponseBody::from(articles))
            .server_err()?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Get Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#get-article
pub async fn get_article(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let authenticated = optionally_auth(&req).transpose()?;

        let state = req.state();
        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;
        let slug = req.param::<String>("slug").client_err()?;

        let article = tx.get_article_by_slug(&slug).await?;

        let profile_entity = tx.get_profile_by_id(article.author_id).await?;

        let profile = if let Some((user_id, _)) = authenticated {
            let following = tx.is_following(profile_entity.user_id, user_id).await?;
            Profile::from(profile_entity).following(following)
        } else {
            Profile::from(profile_entity)
        };

        tx.commit().await.server_err()?;

        let resp = to_json_response(&ArticleResponseBody {
            article: Article::from((article, profile)),
        })?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Create Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#create-article
pub async fn create_article(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        #[derive(Deserialize)]
        struct ArticleRequestBody {
            article: NewArticle,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct NewArticle {
            title: String,
            description: String,
            body: String,
            tag_list: Option<Vec<String>>,
        }

        let (user_id, _) = extract_and_validate_token(&req)?;

        let body: ArticleRequestBody = req.body_json().await.client_err()?;

        let slug = body.article.title.to_kebab_case();
        debug!(
            "Generated slug `{}` from title `{}`",
            slug, body.article.title
        );

        let state = req.state();

        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let (article, profile) = {
            let ArticleRequestBody {
                article:
                    NewArticle {
                        title,
                        description,
                        body,
                        tag_list,
                    },
            } = body;

            let profile = tx.get_profile_by_id(user_id).await?;

            let article = tx
                .create_article(user_id, &title, &slug, &description, &body)
                .await?;

            if let Some(tags) = tag_list.as_ref() {
                tx.create_tags_for_article(article.article_id, tags.as_slice())
                    .await?
            }

            (article, profile)
        };

        tx.commit().await.server_err()?;

        let resp = to_json_response(&ArticleResponseBody {
            article: Article::from((article, profile)),
        })?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Delete Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#delete-article
pub async fn delete_article(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let (user_id, _) = extract_and_validate_token(&req)?;

        let slug = req.param::<String>("slug").client_err()?;

        let state = req.state();
        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let article = tx.get_article_by_slug(&slug).await?;

        if article.author_id != user_id {
            Err(Response::new(403))?
        }

        tx.delete_article(&slug).await?;

        tx.commit().await.server_err()?;

        Ok::<_, Error>(Response::new(200))
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Update the title, description, and/or body of an Article
///
/// [Update Article](https://github.com/gothinkster/realworld/tree/master/api#update-article)
pub async fn update_article(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        #[derive(Deserialize)]
        struct UpdateArticleBody {
            article: ArticleUpdate,
        }

        #[derive(Deserialize)]
        struct ArticleUpdate {
            title: Option<String>,
            description: Option<String>,
            body: Option<String>,
        }

        let (user_id, _) = extract_and_validate_token(&req)?;

        let slug = req.param::<String>("slug").client_err()?;
        let body: UpdateArticleBody = req.body_json().await.client_err()?;

        let state = req.state();
        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let existing = tx.get_article_by_slug(&slug).await?;

        if existing.author_id != user_id {
            Err(Response::new(403))?
        }

        let author = tx.get_profile_by_id(user_id).await?;
        let updates = {
            let UpdateArticleBody {
                article:
                    ArticleUpdate {
                        title,
                        description,
                        body,
                    },
            } = body;
            let new_slug = title
                .as_ref()
                .map_or_else(|| slug, |new_title| new_title.to_kebab_case());

            ArticleEntity {
                title: title.unwrap_or(existing.title),
                slug: new_slug,
                description: description.unwrap_or(existing.description),
                body: body.unwrap_or(existing.body),
                ..existing
            }
        };

        let updated = tx.update_article(&updates).await?;

        let favorites_count = tx.get_favorites_count(&updates.slug).await?;

        tx.commit().await.server_err()?;

        let resp = to_json_response(&ArticleResponseBody {
            article: Article::from((updated, author)).favorites_count(favorites_count),
        })?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Add a comment to an an article
///
/// [Add Comments to an Article](https://github.com/gothinkster/realworld/tree/master/api#add-comments-to-an-article)
pub async fn add_comment(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        #[derive(Deserialize)]
        struct CommentRequestBody {
            comment: NewComment,
        }
        #[derive(Deserialize)]
        struct NewComment {
            body: String,
        }

        let (user_id, _) = extract_and_validate_token(&req)?;
        let slug = req.param::<String>("slug").client_err()?;

        let req_body: CommentRequestBody = req.body_json().await.client_err()?;

        let state = req.state();
        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let _article = tx.get_article_by_slug(&slug).await?;

        let comment_ent = tx
            .create_comment(&slug, user_id, &req_body.comment.body)
            .await?;

        let profile = tx.get_profile_by_id(user_id).await.map(Profile::from)?;

        tx.commit().await.server_err()?;

        let resp_body = CommentResponseBody {
            comment: Comment::from((comment_ent, profile)),
        };

        let resp = to_json_response(&resp_body)?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Get the comments placed on an article
///
/// [Get Comments from an Article](https://github.com/gothinkster/realworld/tree/master/api#get-comments-from-an-article)
pub async fn get_comments(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let authenticated = optionally_auth(&req).transpose()?;

        let slug = req.param::<String>("slug").client_err()?;

        let state = req.state();

        let mut db = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let leader_ids: HashSet<EntityId> = if let Some((user_id, _)) = authenticated {
            HashSet::from_iter(db.get_following(user_id).await?)
        } else {
            HashSet::default()
        };

        let comment_profile_pairs = db.get_comments_on_article(&slug).await?;

        let comments = comment_profile_pairs
            .into_iter()
            .map(|ents| Comment::with_leaders(ents, &leader_ids))
            .collect::<Vec<_>>();
        let resp = to_json_response(&MultipleCommentsResponseBody { comments })?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

pub async fn delete_comment(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let (user_id, _) = extract_and_validate_token(&req)?;

        let slug = req.param::<String>("slug").client_err()?;
        let comment_id = req.param::<EntityId>("comment_id").client_err()?;

        let state = req.state();
        let mut db = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let comment = db.get_comment(&slug, comment_id).await?;

        if comment.author_id != user_id {
            Err(Response::new(403))?
        }

        db.delete_comment(&slug, comment_id).await?;

        db.commit().await.server_err()?;

        Ok::<_, Error>(Response::new(200))
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Favorite Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#favorite-article
pub async fn favorite_article(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    should_favorite(req, true)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

/// Unfavorite Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#favorite-article
pub async fn unfavorite_article(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    should_favorite(req, false)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

async fn should_favorite(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
    should_favorite: bool,
) -> tide::Result<Response> {
    let (user_id, _) = extract_and_validate_token(&req)?;
    let slug = req.param::<String>("slug").client_err()?;

    let state = req.state();
    let mut tx = state
        .conn()
        .and_then(Connection::begin)
        .await
        .server_err()?;

    match should_favorite {
        true => tx.create_favorite(user_id, &slug),
        false => tx.delete_favorite(user_id, &slug),
    }
    .await?;

    let article = tx.get_article_by_slug(&slug).await?;

    let author = tx.get_profile_by_id(article.author_id).await?;

    let favorites_count = tx.get_favorites_count(&slug).await?;

    tx.commit().await.server_err()?;

    let resp = to_json_response(&ArticleResponseBody {
        article: Article {
            favorited: should_favorite,
            favorites_count,
            ..From::from((article, author))
        },
    })?;

    Ok(resp)
}

/// Feed Articles
///
/// https://github.com/gothinkster/realworld/tree/master/api#feed-articles
pub async fn get_feed(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let (user_id, _) = extract_and_validate_token(&req)?;

        let state = req.state();

        let mut db = state.conn().await.server_err()?;

        let leader_ids = db
            .get_following(user_id)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let articles = db
            .get_all_articles()
            .await?
            .into_iter()
            .filter(|(article, _)| leader_ids.contains(&article.author_id))
            .map(|(article, profile)| (article, Profile::from(profile).following(true)))
            .collect::<Vec<_>>();

        let resp = to_json_response(&MultArticlesResponseBody {
            articles: vec![],
            articles_count: articles.len(),
        })?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Get Tags
///
/// https://github.com/gothinkster/realworld/tree/master/api#get-tags
pub async fn get_tags(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideData>>>,
) -> Response {
    async move {
        let state = req.state();
        let mut db = state.conn().await.server_err()?;
        let tags = db.get_tags().await?;
        #[derive(Serialize)]
        struct GetTagsResponse {
            tags: Vec<String>,
        }
        Ok::<_, Error>(to_json_response(&GetTagsResponse { tags })?)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}
