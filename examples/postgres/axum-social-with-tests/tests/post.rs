use sqlx::PgPool;

use sqlx_example_postgres_axum_social::http;

use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use std::borrow::BorrowMut;

use common::{expect_rfc3339_timestamp, expect_uuid, response_json, RequestBuilderExt};

use serde_json::json;

mod common;

#[sqlx::test(fixtures("users"))]
async fn test_create_post(db: PgPool) {
    let mut app = http::app(db);

    // Happy path!
    let mut resp1 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/post").json(json! {
            {
                "auth": {
                    "username": "alice",
                    "password": "rustacean since 2015"
                },
                "content": "This new computer is blazing fast!"
            }
        }))
        .await
        .unwrap();

    assert_eq!(resp1.status(), StatusCode::OK);

    let resp1_json = response_json(&mut resp1).await;

    assert_eq!(resp1_json["username"], "alice");
    assert_eq!(resp1_json["content"], "This new computer is blazing fast!");

    let _post_id = expect_uuid(&resp1_json["postId"]);
    let _created_at = expect_rfc3339_timestamp(&resp1_json["createdAt"]);

    // Incorrect username
    let mut resp2 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/post").json(json! {
            {
                "auth": {
                    "username": "aliceee",
                    "password": "rustacean since 2015"
                },
                "content": "This new computer is blazing fast!"
            }
        }))
        .await
        .unwrap();

    assert_eq!(resp2.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp2_json = response_json(&mut resp2).await;
    assert_eq!(resp2_json["message"], "invalid username/password");

    // Incorrect password
    let mut resp3 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/post").json(json! {
            {
                "auth": {
                    "username": "alice",
                    "password": "rustaceansince2015"
                },
                "content": "This new computer is blazing fast!"
            }
        }))
        .await
        .unwrap();

    assert_eq!(resp3.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp3_json = response_json(&mut resp3).await;
    assert_eq!(resp3_json["message"], "invalid username/password");
}

#[sqlx::test(fixtures("users", "posts"))]
async fn test_list_posts(db: PgPool) {
    // This only has the happy path.
    let mut resp = http::app(db)
        .oneshot(Request::get("/v1/post").empty_body())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let resp_json = response_json(&mut resp).await;

    let posts = resp_json
        .as_array()
        .expect("expected GET /v1/post to return an array");

    assert_eq!(posts.len(), 2);

    assert_eq!(posts[0]["username"], "bob");
    assert_eq!(posts[0]["content"], "@alice is a haxxor");

    let _post_id = expect_uuid(&posts[0]["postId"]);
    let created_at_0 = expect_rfc3339_timestamp(&posts[0]["createdAt"]);

    assert_eq!(posts[1]["username"], "alice");
    assert_eq!(posts[1]["content"], "This new computer is blazing fast!");

    let _post_id = expect_uuid(&posts[1]["postId"]);
    let created_at_1 = expect_rfc3339_timestamp(&posts[1]["createdAt"]);

    assert!(
        created_at_0 > created_at_1,
        "posts must be sorted in descending order"
    );
}
