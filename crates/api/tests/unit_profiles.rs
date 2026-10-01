mod support;

use axum::http::StatusCode;
use tactica_db_model::{
    CreateUnit, CreatedUnit, NewUnit, NewUnitMemberRole, NewUnitMembership, NewUnitRole, NewUser,
    UnitMemberRoleStore, UnitMembershipStore, UnitRoleStore, UnitStore, UserStore,
};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId, UserId};

use support::ApiFixture;

async fn user(api: &ApiFixture, name: &str) -> UserId {
    let id = UserId::new();
    UserStore::create(
        &api.storage,
        NewUser {
            id,
            username: name.to_owned(),
            email: format!("{name}@example.test"),
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

fn new_unit(slug: &str) -> NewUnit {
    NewUnit {
        id: UnitId::new(),
        slug: slug.to_owned(),
        display_name: Some(slug.to_owned()),
        icon_url: None,
        banner_url: None,
        biography: None,
    }
}

async fn unit(api: &ApiFixture, owner_id: UserId, slug: &str) -> CreatedUnit {
    api.storage
        .create_with_defaults(CreateUnit {
            owner_id,
            unit: new_unit(slug),
        })
        .await
        .expect("create unit")
}

async fn patch(
    api: &ApiFixture,
    id: UnitId,
    actor: Option<UserId>,
    body: serde_json::Value,
) -> StatusCode {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let mut request = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/units/{id}"))
        .header("Content-Type", "application/json");
    if let Some(actor) = actor {
        request = request.header(
            "Authorization",
            format!(
                "Bearer {}",
                api.jwt.generate_jwt_for_user(actor).expect("token")
            ),
        );
    }
    api.router
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).expect("request"))
        .await
        .expect("response")
        .status()
}

#[tokio::test(flavor = "multi_thread")]
async fn profile_patch_validates_and_preserves_unrelated_fields() {
    use serde_json::json;
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let created = unit(&api, owner, "original").await;
    let id = created.unit.id;
    unit(&api, owner, "taken").await;
    let mut original = UnitStore::get(&api.storage, id)
        .await
        .expect("get")
        .expect("unit");
    original.icon_url = Some("/saved-icon.png".into());
    let original = UnitStore::update(&api.storage, original)
        .await
        .expect("icon");
    assert_eq!(patch(&api,id,Some(owner),json!({"display_name":" Raven Company ","slug":"raven-company","biography":" Team biography ","banner_url":"https://example.test/banner.png"})).await,StatusCode::OK);
    let saved = UnitStore::get(&api.storage, id)
        .await
        .expect("get")
        .expect("unit");
    assert_eq!(saved.display_name.as_deref(), Some("Raven Company"));
    assert_eq!(saved.biography.as_deref(), Some("Team biography"));
    let (status, response) = api.get(&format!("/api/v1/units/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["display_name"], "Raven Company");
    assert_eq!(saved.icon_url, original.icon_url);
    assert_eq!(saved.owner_id, original.owner_id);
    assert_eq!(saved.created_at, original.created_at);
    assert_eq!(
        patch(&api, id, Some(owner), json!({"slug":"taken"})).await,
        StatusCode::CONFLICT
    );
    for invalid in [
        json!({}),
        json!({"display_name":" "}),
        json!({"slug":"Bad--slug"}),
        json!({"banner_url":"javascript:alert(1)"}),
        json!({"banner_url":"//example.test/image"}),
        json!({"banner_url":"https://user:password@example.test/image"}),
        json!({"biography":"a".repeat(5001)}),
    ] {
        assert_eq!(
            patch(&api, id, Some(owner), invalid).await,
            StatusCode::BAD_REQUEST
        );
    }
    for invalid in [
        json!({"display_name":null}),
        json!({"owner_id":owner}),
        json!({"icon_url":"/other.png"}),
    ] {
        assert!(
            patch(&api, id, Some(owner), invalid)
                .await
                .is_client_error()
        );
    }
    assert_eq!(
        patch(
            &api,
            id,
            Some(owner),
            json!({"biography":null,"banner_url":null})
        )
        .await,
        StatusCode::OK
    );
    let cleared = UnitStore::get(&api.storage, id)
        .await
        .expect("get")
        .expect("unit");
    assert_eq!(cleared.biography, None);
    assert_eq!(cleared.banner_url, None);
    assert_eq!(cleared.slug, "raven-company");
    assert_eq!(cleared.icon_url, original.icon_url);
}

#[tokio::test(flavor = "multi_thread")]
async fn profile_patch_requires_current_manage_unit_permission() {
    use serde_json::json;
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let created = unit(&api, owner, "unit").await;
    let id = created.unit.id;
    let actor = user(&api, "member").await;
    assert_eq!(
        patch(&api, id, None, json!({"biography":"x"})).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        patch(&api, id, Some(actor), json!({"biography":"x"})).await,
        StatusCode::FORBIDDEN
    );
    let member = UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: actor,
            unit_id: id,
            rank_id: created.owner_rank_id,
        },
    )
    .await
    .expect("member");
    assert_eq!(
        patch(&api, id, Some(actor), json!({"biography":"x"})).await,
        StatusCode::FORBIDDEN
    );
    let role_id = RoleId::new();
    UnitRoleStore::create(
        &api.storage,
        NewUnitRole {
            id: role_id,
            unit_id: id,
            display_name: "Profile editor".into(),
            description: None,
            permissions: 2,
        },
    )
    .await
    .expect("role");
    UnitMemberRoleStore::create(
        &api.storage,
        NewUnitMemberRole {
            member_id: member.id,
            role_id,
            unit_id: id,
        },
    )
    .await
    .expect("assign");
    assert_eq!(
        patch(&api, id, Some(actor), json!({"biography":"allowed"})).await,
        StatusCode::OK
    );
    UnitMemberRoleStore::delete(&api.storage, member.id, role_id)
        .await
        .expect("revoke");
    assert_eq!(
        patch(&api, id, Some(actor), json!({"biography":"denied"})).await,
        StatusCode::FORBIDDEN
    );
}
