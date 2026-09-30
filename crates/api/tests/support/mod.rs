use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tactica_auth::{AuthContext, jwt::JwtContext};
use tactica_db::PgConnection;
use tactica_db_schema::migrations::{AsyncMigrationHarness, MIGRATIONS, MigrationHarness};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use tower::ServiceExt;

pub struct ApiFixture {
    _container: testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    pub storage: PgConnection,
    pub router: Router,
    pub jwt: JwtContext,
}

impl ApiFixture {
    pub async fn new() -> Self {
        let container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("start PostgreSQL");
        let host = container.get_host().await.expect("database host");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("database port");
        let storage = PgConnection::new(&format!(
            "postgres://postgres:postgres@{host}:{port}/postgres"
        ))
        .await
        .expect("connect to PostgreSQL");
        AsyncMigrationHarness::new(
            storage
                .pool()
                .get_owned()
                .await
                .expect("migration connection"),
        )
        .run_pending_migrations(MIGRATIONS)
        .expect("run migrations");
        // These keys are disposable test fixtures, never production credentials.
        let jwt = JwtContext::new(
            include_bytes!("jwt-public.pem"),
            include_bytes!("jwt-private.pem"),
        )
        .expect("load test JWT keys");
        let storage_arc = Arc::new(storage.clone());
        let auth = AuthContext::new(storage_arc.clone(), jwt.clone(), "dGVzdC1zYWx0LW9ubHk")
            .expect("test authentication context");
        let router = tactica_api::router(tactica_api::state::ApiState::new(
            storage_arc,
            Arc::new(auth),
        ));
        Self {
            _container: container,
            storage,
            router,
            jwt,
        }
    }

    pub async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        let mut request = Request::builder().uri(path);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let response = self
            .router
            .clone()
            .oneshot(request.body(Body::empty()).expect("request"))
            .await
            .expect("serve request");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response");
        let body = serde_json::from_slice(&bytes).expect("JSON response");
        (status, body)
    }
}
