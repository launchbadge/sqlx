use sqlx::PgPool;

use sqlx_example_postgres_axum_social::http;

use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use std::borrow::BorrowMut;

use common::{response_json, RequestBuilderExt};

use serde_json::json;

mod common;

#[sqlx::test]
async fn test_create_user(db: PgPool) {
    let mut app = http::app(db);

    // Happy path!
    let resp1 = app
        .borrow_mut()
        // We handle JSON objects directly to sanity check the serialization and deserialization
        .oneshot(Request::post("/v1/user").json(json! {{
            "username": "alice",
            "password": "rustacean since 2015"
        }}))
        .await
        .unwrap();

    assert_eq!(resp1.status(), StatusCode::NO_CONTENT);

    // Username taken
    let mut resp2 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/user").json(json! {{
            "username": "alice",
            "password": "uhhh i forgot"
        }}))
        .await
        .unwrap();

    assert_eq!(resp2.status(), StatusCode::CONFLICT);

    let resp2_json = response_json(&mut resp2).await;
    assert_eq!(resp2_json["message"], "username taken");

    // Invalid username
    let mut resp3 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/user").json(json! {{
            "username": "definitely an invalid username",
            "password": "password"
        }}))
        .await
        .unwrap();

    assert_eq!(resp3.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp3_json = response_json(&mut resp3).await;

    assert_eq!(resp3_json["message"], "validation error in request body");
    assert!(
        resp3_json["errors"]["username"].is_array(),
        "errors.username is not an array: {:?}",
        resp3_json
    );

    // Invalid password
    let mut resp4 = app
        .borrow_mut()
        .oneshot(Request::post("/v1/user").json(json! {{
            "username": "bobby123",
            "password": ""
        }}))
        .await
        .unwrap();

    assert_eq!(resp4.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let resp4_json = response_json(&mut resp4).await;

    assert_eq!(resp4_json["message"], "validation error in request body");
    assert!(
        resp4_json["errors"]["password"].is_array(),
        "errors.password is not an array: {:?}",
        resp4_json
    );
}
