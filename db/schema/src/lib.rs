//! Postgres schema for Tactica.

pub mod schema;


#[cfg(feature = "migrations")]
pub mod migrations {
    use diesel_migrations::{EmbeddedMigrations, embed_migrations};
    pub use diesel_migrations::MigrationHarness;
    pub use diesel_async::AsyncMigrationHarness;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
}
