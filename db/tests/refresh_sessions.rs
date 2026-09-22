mod support;

use chrono::{Duration, Utc};
use tactica_db::PgConnection;
use tactica_db_model::{NewRefreshSession, NewUser, RefreshSessionStore, StoreError, UserStore};
use tactica_uuid_kinds::{SessionId, UserId};

async fn user(storage: &PgConnection) -> UserId {
    let id = UserId::new();
    UserStore::create(
        storage,
        NewUser {
            id,
            username: "refresh-user".to_owned(),
            email: "refresh@example.test".to_owned(),
            display_name: None,
            icon_url: None,
            banner_url: None,
            biography: None,
            is_active: true,
            is_superuser: false,
            password_hash: None,
            totp_secret: None,
        },
    )
    .await
    .expect("create user");
    id
}

fn hash(value: u8) -> String {
    format!("{value:064x}")
}

async fn session(storage: &PgConnection, user_id: UserId, token: u8, days: i64) {
    storage
        .create_refresh_session(NewRefreshSession {
            id: SessionId::new(),
            user_id,
            token_hash: hash(token),
            expires_at: Utc::now() + Duration::days(days),
        })
        .await
        .expect("create session");
}

#[tokio::test(flavor = "multi_thread")]
async fn rotation_detects_replay_and_logout_revokes_only_its_session() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let user_id = user(&storage).await;
    session(&storage, user_id, 1, 30).await;
    session(&storage, user_id, 10, 30).await;
    assert_eq!(
        storage
            .rotate_refresh_token(hash(1), hash(2))
            .await
            .expect("rotate"),
        Some(user_id)
    );
    assert_eq!(
        storage
            .rotate_refresh_token(hash(2), hash(3))
            .await
            .expect("rotate again"),
        Some(user_id)
    );
    assert_eq!(
        storage
            .rotate_refresh_token(hash(1), hash(4))
            .await
            .expect("replay"),
        None
    );
    assert_eq!(
        storage
            .rotate_refresh_token(hash(3), hash(5))
            .await
            .expect("revoked descendant"),
        None
    );
    assert_eq!(
        storage
            .rotate_refresh_token(hash(10), hash(11))
            .await
            .expect("independent session"),
        Some(user_id)
    );
    storage
        .revoke_refresh_session(hash(10))
        .await
        .expect("logout with consumed token");
    storage
        .revoke_refresh_session(hash(10))
        .await
        .expect("repeat logout");
    storage
        .revoke_refresh_session(hash(99))
        .await
        .expect("unknown logout");
    assert_eq!(
        storage
            .rotate_refresh_token(hash(11), hash(12))
            .await
            .expect("revoked session"),
        None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn expired_unknown_inactive_and_deleted_users_cannot_refresh() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let user_id = user(&storage).await;
    session(&storage, user_id, 1, -1).await;
    assert_eq!(
        storage
            .rotate_refresh_token(hash(1), hash(2))
            .await
            .expect("expired"),
        None
    );
    assert_eq!(
        storage
            .rotate_refresh_token(hash(99), hash(2))
            .await
            .expect("unknown"),
        None
    );
    session(&storage, user_id, 3, 30).await;
    let mut user = UserStore::get(&storage, user_id)
        .await
        .expect("get user")
        .expect("user exists");
    user.is_active = false;
    let mut user = UserStore::update(&storage, user).await.expect("deactivate");
    assert_eq!(
        storage
            .rotate_refresh_token(hash(3), hash(4))
            .await
            .expect("inactive"),
        None
    );
    user.is_active = true;
    UserStore::update(&storage, user).await.expect("reactivate");
    assert_eq!(
        storage
            .rotate_refresh_token(hash(3), hash(4))
            .await
            .expect("revocation persists"),
        None
    );
    session(&storage, user_id, 5, 30).await;
    UserStore::delete(&storage, user_id)
        .await
        .expect("delete user");
    assert_eq!(
        storage
            .rotate_refresh_token(hash(5), hash(6))
            .await
            .expect("deleted"),
        None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_refresh_has_one_winner_and_replay_revokes_its_replacement() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let user_id = user(&storage).await;
    session(&storage, user_id, 1, 30).await;
    let (first, second) = tokio::join!(
        storage.rotate_refresh_token(hash(1), hash(2)),
        storage.rotate_refresh_token(hash(1), hash(3)),
    );
    let results = [
        first.expect("first rotation"),
        second.expect("second rotation"),
    ];
    assert_eq!(results.iter().filter(|result| result.is_some()).count(), 1);
    for token in [2, 3] {
        assert_eq!(
            storage
                .rotate_refresh_token(hash(token), hash(4))
                .await
                .expect("replacement is unusable"),
            None
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_replacement_insert_rolls_back_consumption() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let user_id = user(&storage).await;
    session(&storage, user_id, 1, 30).await;
    session(&storage, user_id, 2, 30).await;
    assert!(matches!(
        storage.rotate_refresh_token(hash(1), hash(2)).await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        storage
            .rotate_refresh_token(hash(1), hash(3))
            .await
            .expect("retry after rollback"),
        Some(user_id)
    );
}
