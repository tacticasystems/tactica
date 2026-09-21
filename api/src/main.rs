use std::{net::SocketAddr, sync::Arc};

use tactica_api::{router, state::ApiState};
use tactica_db_schema::migrations::{AsyncMigrationHarness, MIGRATIONS, MigrationHarness};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // TODO: replace with clap
    let database_url = std::env::var("TACTICA_DB_URL").expect("TACTICA_DB_URL must be set");
    let conn = tactica_db::PgConnection::new(&database_url)
        .await
        .expect("Failed to connect to database");

    let should_run_migrations = std::env::var("TACTICA_RUN_MIGRATIONS")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .expect("Failed to parse TACTICA_RUN_MIGRATIONS");

    if should_run_migrations {
        let mut harness = AsyncMigrationHarness::new(
            conn
                .pool()
                .get_owned()
                .await
                .expect("Failed to get connection from pool")
        );

        harness.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        return;
    }

    let jwt_key_pub_path = std::env::var("TACTICA_JWT_KEY_PUB_PATH")
        .expect("TACTICA_JWT_KEY_PUB_PATH must be set")
        .parse()
        .expect("Failed to parse TACTICA_JWT_KEY_PUB_PATH");

    let jwt_key_priv_path = std::env::var("TACTICA_JWT_KEY_PRIV_PATH")
        .expect("TACTICA_JWT_KEY_PRIV_PATH must be set")
        .parse()
        .expect("Failed to parse TACTICA_JWT_KEY_PRIV_PATH");

    let jwt_context = tactica_auth::jwt::JwtContext::from_files(
        &jwt_key_pub_path,
        &jwt_key_priv_path,
    )
        .expect("Failed to create JWT context");

    let auth_context = tactica_auth::AuthContext::new(
        Arc::new(conn.clone()),
        jwt_context,
        std::env::var("TACTICA_AUTH_SALT")
            .expect("TACTICA_AUTH_SALT must be set")
    )
        .expect("Failed to create AuthContext");

    let listen_addr: SocketAddr = std::env::var("TACTICA_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("Failed to parse LISTEN_ADDR");

    let state = ApiState::new(Arc::new(conn), Arc::new(auth_context));

    let listener = TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to listen address");

    let router = router(state);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
