use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::db::model::*;

/// An article
///
/// [API Spec](https://github.com/gothinkster/realworld/tree/master/api#single-article)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::api) struct Article {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub body: String,
    pub author: Profile,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tag_list: Vec<String>,
    pub favorited: bool,
    pub favorites_count: usize,
}

impl Article {
    /// Create an article with the author.following field populated
    pub fn with_following(
        entities: (ArticleEntity, ProfileEntity),
        leader_ids: &HashSet<EntityId>,
    ) -> Self {
        let is_following = leader_ids.contains(&entities.1.user_id);
        let mut article = Article::from(entities);
        article.author.following = is_following;
        article
    }

    /// Set the favorites_count
    pub fn favorites_count(self, favorites_count: usize) -> Self {
        Article {
            favorites_count,
            ..self
        }
    }
}

impl From<(ArticleEntity, ProfileEntity)> for Article {
    fn from(entities: (ArticleEntity, ProfileEntity)) -> Self {
        let article = entities.0;
        let author = Profile::from(entities.1);
        (article, author).into()
    }
}

impl From<(ArticleEntity, Profile)> for Article {
    fn from(entities: (ArticleEntity, Profile)) -> Self {
        let ArticleEntity {
            title,
            slug,
            description,
            body,
            created_at,
            updated_at,
            ..
        } = entities.0;
        let author = entities.1;

        Article {
            title,
            slug,
            description,
            body,
            author,
            created_at,
            updated_at,
            tag_list: vec![],
            favorited: false,
            favorites_count: 0,
        }
    }
}

/// A profile for a User
///
/// [API Spec](https://github.com/gothinkster/realworld/tree/master/api#profile)
#[derive(Default, serde::Serialize)]
pub(in crate::api) struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

impl Profile {
    pub fn following(self, following: bool) -> Self {
        Profile { following, ..self }
    }
}

impl From<ProfileEntity> for Profile {
    fn from(ent: ProfileEntity) -> Self {
        let ProfileEntity {
            username,
            bio,
            image,
            ..
        } = ent;

        Profile {
            username,
            bio,
            image,
            following: false,
        }
    }
}
