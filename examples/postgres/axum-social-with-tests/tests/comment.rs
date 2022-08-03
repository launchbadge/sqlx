use sqlx::PgPool;

use sqlx_example_postgres_axum_social::http;

use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use std::borrow::BorrowMut;

use common::{expect_rfc3339_timestamp, expect_uuid, response_json, RequestBuilderExt};

use serde_json::json;

mod common;

#[sqlx::test(fixtures("users", "posts"))]
async fn test_create_comment(db: PgPool) {
    let mut app = http::app(db);

    // Happy path!
    let mut resp1 = app
        .borrow_mut()
        .oneshot(
            Request::post("/v1/post/d9ca2672-24c5-4442-b32f-cd717adffbaa/comment").json(json! {
                {
                    "auth": {
                        "username": "bob",
                        "password": "pro gamer 1990"
                    },
                    "content": "lol bet ur still bad, 1v1 me"
                }
            }),
        )
        .await
        .unwrap();

    assert_eq!(resp1.status(), StatusCode::OK);

    let resp1_json = response_json(&mut resp1).await;

    assert_eq!(resp1_json["username"], "bob");
    assert_eq!(resp1_json["content"], "lol bet ur still bad, 1v1 me");

    let _comment_id = expect_uuid(&resp1_json["commentId"]);

    let _created_at = expect_rfc3339_timestamp(&resp1_json["createdAt"]);

    // Incorrect username
    let mut resp2 = app
        .borrow_mut()
        .oneshot(
            Request::post("/v1/post/d9ca2672-24c5-4442-b32f-cd717adffbaa/comment").json(json! {
                {
                    "auth": {
                        "username": "bobbbbbb",
                        "password": "pro gamer 1990"
                    },
                    "content": "lol bet ur still bad, 1v1 me"
                }
            }),
        )
        .await
        .unwrap();

    assert_eq!(resp2.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp2_json = response_json(&mut resp2).await;
    assert_eq!(resp2_json["message"], "invalid username/password");

    // Incorrect password
    let mut resp3 = app
        .borrow_mut()
        .oneshot(
            Request::post("/v1/post/d9ca2672-24c5-4442-b32f-cd717adffbaa/comment").json(json! {
                {
                    "auth": {
                        "username": "bob",
                        "password": "pro    gamer     1990"
                    },
                    "content": "lol bet ur still bad, 1v1 me"
                }
            }),
        )
        .await
        .unwrap();

    assert_eq!(resp3.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp3_json = response_json(&mut resp3).await;
    assert_eq!(resp3_json["message"], "invalid username/password");
}

#[sqlx::test(fixtures("users", "posts", "comments"))]
async fn test_list_comments(db: PgPool) {
    let mut app = http::app(db);

    // This only has the happy path.
    let mut resp = app
        .borrow_mut()
        .oneshot(Request::get("/v1/post/d9ca2672-24c5-4442-b32f-cd717adffbaa/comment").empty_body())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let resp_json = response_json(&mut resp).await;

    let comments = resp_json
        .as_array()
        .expect("expected request to return an array");

    assert_eq!(comments.len(), 2);

    assert_eq!(comments[0]["username"], "bob");
    assert_eq!(comments[0]["content"], "lol bet ur still bad, 1v1 me");

    let _comment_id = expect_uuid(&comments[0]["commentId"]);
    let created_at_0 = expect_rfc3339_timestamp(&comments[0]["createdAt"]);

    assert_eq!(comments[1]["username"], "alice");
    assert_eq!(comments[1]["content"], "you're on!");

    let _comment_id = expect_uuid(&comments[1]["commentId"]);
    let created_at_1 = expect_rfc3339_timestamp(&comments[1]["createdAt"]);

    assert!(
        created_at_0 < created_at_1,
        "comments must be assorted in ascending order"
    );

    let mut resp = app
        .borrow_mut()
        .oneshot(Request::get("/v1/post/7e3d4d16-a35e-46ba-8223-b4f1debbfbfe/comment").empty_body())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let resp_json = response_json(&mut resp).await;

    let comments = resp_json
        .as_array()
        .expect("expected request to return an array");

    assert_eq!(comments.len(), 1);

    assert_eq!(comments[0]["username"], "alice");
    assert_eq!(comments[0]["content"], "lol you're just mad you lost :P");

    let _comment_id = expect_uuid(&comments[0]["commentId"]);
    let _created_at = expect_rfc3339_timestamp(&comments[0]["createdAt"]);
}
