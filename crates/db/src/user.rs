use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{ListPagination, NewUser, StoreError, User, UserFilter, UserStore};
use tactica_db_schema::schema::users;
use tactica_uuid_kinds::UserId;

use crate::PgConnection;

#[async_trait]
impl UserStore for PgConnection {
    async fn list(
        &self,
        filter: UserFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<User>, StoreError> {
        let mut query = users::table.into_boxed();

        let UserFilter {
            id,
            email,
            username,
        } = filter;

        if let Some(id) = id {
            query = query.filter(users::id.eq_any(id.into_iter().map(|id| *id.as_uuid())));
        }

        if let Some(email) = email {
            query = query.filter(users::email.eq_any(email));
        }

        if let Some(username) = username {
            query = query.filter(users::username.eq_any(username));
        }

        let users = query
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .get_results::<User>(&mut self.conn().await?)
            .await?;

        Ok(users)
    }

    async fn get(&self, id: UserId) -> Result<Option<User>, StoreError> {
        let users = UserStore::list(
            self,
            UserFilter::default().id(vec![id]),
            &ListPagination::default().limit(1),
        )
        .await?;
        Ok(users.into_iter().next())
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, StoreError> {
        let users = UserStore::list(
            self,
            UserFilter::default().username(vec![username.to_string()]),
            &ListPagination::default().limit(1),
        )
        .await?;
        Ok(users.into_iter().next())
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, StoreError> {
        let users = UserStore::list(
            self,
            UserFilter::default().email(vec![email.to_string()]),
            &ListPagination::default().limit(1),
        )
        .await?;
        Ok(users.into_iter().next())
    }

    async fn create(&self, user: NewUser) -> Result<User, StoreError> {
        insert_into(users::dsl::users)
            .values((
                users::id.eq(user.id.as_uuid()),
                users::username.eq(user.username),
                users::email.eq(user.email),
                users::display_name.eq(user.display_name),
                users::icon_url.eq(user.icon_url),
                users::banner_url.eq(user.banner_url),
                users::biography.eq(user.biography),
                users::is_active.eq(user.is_active),
                users::is_superuser.eq(user.is_superuser),
                users::password_hash.eq(user.password_hash),
                users::totp_secret.eq(user.totp_secret),
            ))
            .get_result::<User>(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, user: User) -> Result<User, StoreError> {
        update(users::dsl::users)
            .filter(users::id.eq(user.id.as_uuid()))
            .set((
                users::username.eq(user.username),
                users::email.eq(user.email),
                users::display_name.eq(user.display_name),
                users::icon_url.eq(user.icon_url),
                users::banner_url.eq(user.banner_url),
                users::biography.eq(user.biography),
                users::is_active.eq(user.is_active),
                users::is_superuser.eq(user.is_superuser),
                users::password_hash.eq(user.password_hash),
                users::totp_secret.eq(user.totp_secret),
            ))
            .get_result::<User>(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: UserId) -> Result<(), StoreError> {
        let _ = delete(users::dsl::users)
            .filter(users::id.eq(id.as_uuid()))
            .execute(&mut self.conn().await?)
            .await?;

        Ok(())
    }
}
