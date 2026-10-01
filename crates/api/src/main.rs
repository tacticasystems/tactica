use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use clap::Parser;
use tactica_api::{router, state::ApiState};
use tactica_db_schema::migrations::{AsyncMigrationHarness, MIGRATIONS, MigrationHarness};
use tokio::net::TcpListener;

#[derive(Parser)]
struct Args {
    #[clap(long, short, env = "TACTICA_DB_URL")]
    database_url: String,

    #[clap(long, short, env = "TACTICA_RUN_MIGRATIONS", default_value = "false")]
    run_migrations: bool,

    #[clap(long, env = "TACTICA_JWT_KEY_PUB_PATH")]
    jwt_key_pub_path: PathBuf,

    #[clap(long, env = "TACTICA_JWT_KEY_PRIV_PATH")]
    jwt_key_priv_path: PathBuf,

    #[clap(long, short, env = "TACTICA_AUTH_SALT")]
    auth_salt: String,

    #[clap(
        long,
        short,
        env = "TACTICA_LISTEN_ADDR",
        default_value = "0.0.0.0:8080"
    )]
    listen_addr: SocketAddr,

    #[clap(
        long,
        env = "TACTICA_FILE_BACKEND",
        default_value = "filesystem",
        value_enum
    )]
    file_backend: FileBackend,
    #[clap(long, env = "TACTICA_FILE_ROOT", default_value = "./uploads")]
    file_root: PathBuf,
    #[clap(long, env = "TACTICA_S3_BUCKET", required_if_eq("file_backend", "s3"))]
    s3_bucket: Option<String>,
    #[clap(long, env = "TACTICA_S3_REGION", required_if_eq("file_backend", "s3"))]
    s3_region: Option<String>,
}

#[derive(Clone, clap::ValueEnum)]
enum FileBackend {
    Filesystem,
    S3,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let conn = tactica_db::PgConnection::new(&args.database_url)
        .await
        .expect("Failed to connect to database");

    if args.run_migrations {
        tracing::info!("running migrations...");

        let mut harness = AsyncMigrationHarness::new(
            conn.pool()
                .get_owned()
                .await
                .expect("Failed to get connection from pool"),
        );

        harness
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        tracing::info!("migrations completed successfully");

        return;
    }

    let jwt_context =
        tactica_auth::jwt::JwtContext::from_files(&args.jwt_key_pub_path, &args.jwt_key_priv_path)
            .expect("Failed to create JWT context");

    let auth_context =
        tactica_auth::AuthContext::new(Arc::new(conn.clone()), jwt_context, &args.auth_salt)
            .expect("Failed to create AuthContext");

    let files: Arc<dyn tactica_files::FileStorage> = match args.file_backend {
        FileBackend::Filesystem => {
            std::fs::create_dir_all(&args.file_root).expect("Failed to create upload directory");
            Arc::new(
                tactica_files::FilesystemStorage::new(&args.file_root)
                    .expect("Failed to initialize file storage"),
            )
        }
        FileBackend::S3 => Arc::new(
            tactica_files::S3Storage::from_env(
                args.s3_bucket.as_deref().expect("S3 bucket is required"),
                args.s3_region.as_deref().expect("S3 region is required"),
            )
            .expect("Failed to initialize S3 storage"),
        ),
    };
    let state = ApiState::new(Arc::new(conn), Arc::new(auth_context)).with_file_storage(files);

    let listener = TcpListener::bind(args.listen_addr)
        .await
        .expect("Failed to bind to listen address");

    tracing::info!(addr = ?args.listen_addr, "server listening");

    let router = router(state);
    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
