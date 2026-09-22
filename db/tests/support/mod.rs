use tactica_db_schema::migrations::{AsyncMigrationHarness, MIGRATIONS, MigrationHarness};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};

/// A migrated, disposable PostgreSQL database for integration tests.
///
/// Keeping the container in this value's ownership keeps PostgreSQL alive for
/// as long as the test fixture is used. Other database integration tests can
/// share this support module and call `connect` for the application's pool.
pub struct DatabaseFixture {
    _container: testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    database_url: String,
}

impl DatabaseFixture {
    pub async fn new() -> Self {
        let container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("start PostgreSQL test container");
        let host = container.get_host().await.expect("get PostgreSQL host");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("get PostgreSQL port");
        let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

        let pool = tactica_db::PgConnection::new(&database_url)
            .await
            .expect("create PostgreSQL migration pool");
        let migration_connection = pool.pool().get_owned().await.expect("get migration connection");
        AsyncMigrationHarness::new(migration_connection)
            .run_pending_migrations(MIGRATIONS)
            .expect("run database migrations");

        Self { _container: container, database_url }
    }

    pub async fn connect(&self) -> tactica_db::PgConnection {
        tactica_db::PgConnection::new(&self.database_url)
            .await
            .expect("create PostgreSQL connection pool")
    }
}
