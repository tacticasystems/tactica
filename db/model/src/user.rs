use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::users;
use tactica_uuid_kinds::UserId;

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = users)]
#[partial(NewUser)]
pub struct User {
    pub id: UserId,

    pub username: String,
    pub email: String,

    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,

    pub is_active: bool,
    pub is_superuser: bool,

    pub password_hash: Option<String>,
    pub totp_secret: Option<String>,

    #[partial(NewUser(skip))]
    pub created_at: DateTime<Utc>,
    #[partial(NewUser(skip))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UserFilter {
    pub id: Option<Vec<UserId>>,
    pub email: Option<Vec<String>>,
    pub username: Option<Vec<String>>,
}

impl UserFilter {
    pub fn id(mut self, id: Vec<UserId>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn email(mut self, email: Vec<String>) -> Self {
        self.email = Some(email);
        self
    }

    pub fn username(mut self, username: Vec<String>) -> Self {
        self.username = Some(username);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UserStore {
    /// Lists users in the database, filtered and paginated by the given
    /// arguments.
    async fn list(
        &self,
        filter: UserFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<User>, StoreError>;

    /// Gets a user by their ID.
    async fn get(&self, id: UserId) -> Result<Option<User>, StoreError>;

    /// Gets a user by their username.
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, StoreError>;

    /// Gets a user by their email address.
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, StoreError>;

    /// Creates a new user in the database.
    async fn create(&self, user: NewUser) -> Result<User, StoreError>;

    /// Updates an existing user.
    ///
    /// Uses the user's ID to match which user to update.
    async fn update(&self, user: User) -> Result<User, StoreError>;

    /// Deletes a user by their ID.
    async fn delete(&self, id: UserId) -> Result<(), StoreError>;
}
