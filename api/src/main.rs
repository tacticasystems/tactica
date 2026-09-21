use std::{net::SocketAddr, sync::Arc};

use tactica_api::{router, state::ApiState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // TODO: replace with clap
    let database_url = std::env::var("TACTICA_DB_URL").expect("TACTICA_DB_URL must be set");
    let conn = tactica_db::PgConnection::new(&database_url)
        .await
        .expect("Failed to connect to database");

    let listen_addr: SocketAddr = std::env::var("TACTICA_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("Failed to parse LISTEN_ADDR");

    let state = ApiState::new(Arc::new(conn));

    let listener = TcpListener::bind(listen_addr)
        .await
        .expect("Failed to bind to listen address");

    let router = router(state);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
