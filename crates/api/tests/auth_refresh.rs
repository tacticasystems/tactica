mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tactica_api_types::v1::auth::{LoginResponse, RegisterResponse};
use tactica_db_model::UserStore;
use tower::ServiceExt;

use support::ApiFixture;

async fn post(api: &ApiFixture, path: &str, body: Value) -> (StatusCode, Value) {
    let response = api
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    if status == StatusCode::OK && matches!(path, "/api/v1/auth/login" | "/api/v1/auth/refresh") {
        assert_eq!(
            response
                .headers()
                .get("cache-control")
                .expect("cache policy"),
            "no-store"
        );
        assert_eq!(
            response
                .headers()
                .get("pragma")
                .expect("legacy cache policy"),
            "no-cache"
        );
    }
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("JSON")
    };
    (status, body)
}

async fn register(api: &ApiFixture) -> RegisterResponse {
    let (status, body) = post(
        api,
        "/api/v1/auth/register",
        json!({
            "username": "refresh-user", "email": "refresh@example.test",
            "password": "test-only-password", "password_confirm": "test-only-password",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_value(body).expect("registered user")
}

async fn login(api: &ApiFixture) -> LoginResponse {
    let (status, body) = post(
        api,
        "/api/v1/auth/login",
        json!({
            "username": "refresh-user", "password": "test-only-password",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_value(body).expect("token pair")
}

#[tokio::test(flavor = "multi_thread")]
async fn login_refresh_and_logout_work_without_an_access_token() {
    let api = ApiFixture::new().await;
    let user = register(&api).await;
    let initial = login(&api).await;
    assert_eq!(initial.token_type, "Bearer");
    assert_eq!(initial.expires_in, 300);
    assert!(initial.refresh_token.starts_with("rt_"));
    assert_eq!(initial.refresh_token.len(), 67);
    let claims = api
        .jwt
        .validate_jwt(&initial.access_token)
        .expect("valid access JWT");
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.exp - claims.nbf, initial.expires_in);
    assert_eq!(
        api.get("/api/v1/auth/me", Some(&initial.access_token))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        api.get("/api/v1/auth/me", Some(&initial.refresh_token))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );

    let (status, body) = post(
        &api,
        "/api/v1/auth/refresh",
        json!({"refresh_token": initial.refresh_token}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let refreshed: LoginResponse = serde_json::from_value(body).expect("refreshed pair");
    assert_ne!(refreshed.refresh_token, initial.refresh_token);
    assert_eq!(refreshed.expires_in, 300);
    assert_eq!(
        api.get("/api/v1/auth/me", Some(&refreshed.access_token))
            .await
            .0,
        StatusCode::OK
    );
    // A consumed ancestor can still identify the session for logout.
    for _ in 0..2 {
        assert_eq!(
            post(
                &api,
                "/api/v1/auth/logout",
                json!({"refresh_token": initial.refresh_token})
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        post(
            &api,
            "/api/v1/auth/refresh",
            json!({"refresh_token": refreshed.refresh_token})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    // Logout revokes refresh credentials; access JWTs retain their short lifetime.
    assert_eq!(
        api.get("/api/v1/auth/me", Some(&refreshed.access_token))
            .await
            .0,
        StatusCode::OK
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn replay_revokes_replacement_but_not_another_login() {
    let api = ApiFixture::new().await;
    register(&api).await;
    let first = login(&api).await;
    let separate = login(&api).await;
    assert_ne!(first.refresh_token, separate.refresh_token);
    let (_, body) = post(
        &api,
        "/api/v1/auth/refresh",
        json!({"refresh_token": first.refresh_token}),
    )
    .await;
    let replacement: LoginResponse = serde_json::from_value(body).expect("replacement");
    for token in [&first.refresh_token, &replacement.refresh_token] {
        let (status, body) = post(
            &api,
            "/api/v1/auth/refresh",
            json!({"refresh_token": token}),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            body.get("code").and_then(Value::as_str),
            Some("unauthorized")
        );
    }
    assert_eq!(
        post(
            &api,
            "/api/v1/auth/refresh",
            json!({"refresh_token": separate.refresh_token})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_credentials_and_inactive_accounts_cannot_get_tokens() {
    let api = ApiFixture::new().await;
    let registered = register(&api).await;
    let initial = login(&api).await;
    for token in [
        String::new(),
        "garbage".to_owned(),
        format!("rt_{}", "0".repeat(64)),
        initial.access_token,
    ] {
        assert_eq!(
            post(
                &api,
                "/api/v1/auth/refresh",
                json!({"refresh_token": token})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        post(
            &api,
            "/api/v1/auth/logout",
            json!({"refresh_token": "unknown"})
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    for (username, password) in [("refresh-user", "wrong"), ("missing", "test-only-password")] {
        assert_eq!(
            post(
                &api,
                "/api/v1/auth/login",
                json!({"username": username, "password": password})
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    let mut user = UserStore::get(&api.storage, registered.id)
        .await
        .expect("get user")
        .expect("user exists");
    user.is_active = false;
    UserStore::update(&api.storage, user)
        .await
        .expect("deactivate");
    assert_eq!(
        post(
            &api,
            "/api/v1/auth/login",
            json!({"username": "refresh-user", "password": "test-only-password"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post(
            &api,
            "/api/v1/auth/refresh",
            json!({"refresh_token": initial.refresh_token})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
