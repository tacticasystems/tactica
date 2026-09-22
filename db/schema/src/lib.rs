//! Postgres schema for Tactica.

pub mod schema;

#[cfg(feature = "migrations")]
pub mod migrations {
    pub use diesel_async::AsyncMigrationHarness;
    pub use diesel_migrations::MigrationHarness;
    use diesel_migrations::{EmbeddedMigrations, embed_migrations};
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
}
