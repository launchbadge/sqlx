use crate::db::model::ProvideArticle;
use tide::{Request, Response};

struct Article {
    title: String,
    description: String,
    body: String,
    // ...etc...
}

/// List Articles
///
/// https://github.com/gothinkster/realworld/tree/master/api#list-articles
pub async fn list_articles(req: Request<impl ProvideArticle>) -> Response {
    unimplemented!()
}

/// Get Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#get-article
pub async fn get_article(req: Request<impl ProvideArticle>) -> Response {
    unimplemented!()
}

/// Create Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#create-article
pub async fn create_article(req: Request<impl ProvideArticle>) -> Response {
    unimplemented!()
}

/// Delete Article
///
/// https://github.com/gothinkster/realworld/tree/master/api#delete-article
///
/// /api/articles/:slug
pub async fn update_article(req: Request<impl ProvideArticle>) -> Response {
    unimplemented!()
}
