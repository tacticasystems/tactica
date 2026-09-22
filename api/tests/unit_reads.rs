mod support;

use axum::http::StatusCode;
use tactica_api_types::v1::{
    members::{ListMemberRolesResponse, ListMembersResponse},
    ranks::ListRanksResponse,
    roles::ListRolesResponse,
    units::{ListUnitsResponse, UnitSummary},
};
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

async fn role(api: &ApiFixture, unit_id: UnitId, member_id: MemberId, name: &str) -> RoleId {
    let id = RoleId::new();
    UnitRoleStore::create(
        &api.storage,
        NewUnitRole {
            id,
            unit_id,
            display_name: name.to_owned(),
            description: None,
            permissions: 1,
        },
    )
    .await
    .expect("create role");
    UnitMemberRoleStore::create(
        &api.storage,
        NewUnitMemberRole {
            member_id,
            role_id: id,
            unit_id,
        },
    )
    .await
    .expect("assign role");
    id
}

#[tokio::test(flavor = "multi_thread")]
async fn public_units_include_total_member_counts_even_when_paginated() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let populated = unit(&api, owner, "populated").await;
    let other = unit(&api, owner, "other").await;
    let empty = UnitStore::create(&api.storage, owner, new_unit("empty"))
        .await
        .expect("empty unit");
    let additional_member = UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: user(&api, "second").await,
            unit_id: populated.unit.id,
            rank_id: populated.owner_rank_id,
        },
    )
    .await
    .expect("second member");

    let (status, body) = api.get("/api/v1/units", None).await;
    assert_eq!(status, StatusCode::OK);
    let listing: ListUnitsResponse = serde_json::from_value(body).expect("unit listing");
    assert_eq!(listing.units.len(), 3);
    for (id, count) in [(populated.unit.id, 2), (other.unit.id, 1), (empty.id, 0)] {
        assert_eq!(
            listing
                .units
                .iter()
                .find(|unit| unit.id == id)
                .expect("listed unit")
                .member_count,
            count
        );
        let (status, body) = api.get(&format!("/api/v1/units/{id}"), None).await;
        assert_eq!(status, StatusCode::OK);
        let detail: UnitSummary = serde_json::from_value(body).expect("unit detail");
        assert_eq!(detail.id, id);
        assert_eq!(detail.member_count, count);
    }
    let (status, body) = api.get("/api/v1/units?offset=0&limit=1", None).await;
    assert_eq!(status, StatusCode::OK);
    let page: ListUnitsResponse = serde_json::from_value(body).expect("unit page");
    assert_eq!(page.units.len(), 1);
    assert_eq!(page.units.first().expect("first unit").member_count, 2);
    let (status, body) = api.get("/api/v1/units?offset=1&limit=1", None).await;
    assert_eq!(status, StatusCode::OK);
    let page: ListUnitsResponse = serde_json::from_value(body).expect("second page");
    assert_eq!(page.units.first().expect("second unit").id, other.unit.id);
    assert_eq!(page.units.first().expect("second unit").member_count, 1);

    UnitMembershipStore::delete(&api.storage, additional_member.id)
        .await
        .expect("remove member");
    let (_, body) = api
        .get(&format!("/api/v1/units/{}", populated.unit.id), None)
        .await;
    let detail: UnitSummary = serde_json::from_value(body).expect("updated count");
    assert_eq!(detail.member_count, 1);
    assert_eq!(
        api.get(&format!("/api/v1/units/{}", UnitId::new()), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let (_, body) = api.get("/api/v1/units?offset=100", None).await;
    let page: ListUnitsResponse = serde_json::from_value(body).expect("empty page");
    assert!(page.units.is_empty());
    for query in ["offset=-1", "limit=0", "limit=101"] {
        assert_eq!(
            api.get(&format!("/api/v1/units?{query}"), None).await.0,
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn unit_members_can_read_only_the_requested_units_roster_data() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let first = unit(&api, owner, "first").await;
    let second = unit(&api, owner, "second").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("token");
    let first_role = role(&api, first.unit.id, first.owner_membership_id, "First").await;
    let second_role = role(&api, first.unit.id, first.owner_membership_id, "Second").await;
    role(&api, second.unit.id, second.owner_membership_id, "Other").await;
    let base = format!("/api/v1/units/{}", first.unit.id);

    let (status, body) = api.get(&format!("{base}/members"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let members: ListMembersResponse = serde_json::from_value(body).expect("members");
    assert_eq!(members.members.len(), 1);
    let member = members.members.first().expect("owner membership");
    assert_eq!(member.id, first.owner_membership_id);
    assert_eq!(member.user_id, owner);
    assert_eq!(member.rank_id, first.owner_rank_id);

    let (status, body) = api.get(&format!("{base}/ranks"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let ranks: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    assert_eq!(ranks.ranks.len(), 2);
    assert!(ranks.ranks.iter().all(|rank| rank.unit_id == first.unit.id));
    assert!(
        ranks
            .ranks
            .iter()
            .any(|rank| rank.id == first.owner_rank_id)
    );

    let (status, body) = api.get(&format!("{base}/roles"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let roles: ListRolesResponse = serde_json::from_value(body).expect("roles");
    assert_eq!(roles.roles.len(), 4);
    assert!(roles.roles.iter().all(|role| role.unit_id == first.unit.id));
    assert_eq!(
        roles.roles.first().expect("highest role").kind,
        "administrator"
    );
    assert_eq!(
        roles
            .roles
            .iter()
            .find(|role| role.kind == "custom")
            .expect("first custom role")
            .id,
        first_role
    );
    let everyone_id = roles
        .roles
        .iter()
        .find(|role| role.kind == "everyone")
        .expect("Everyone role")
        .id;

    let assignments = format!("{base}/members/{}/roles", first.owner_membership_id);
    let (status, body) = api.get(&assignments, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let roles: ListMemberRolesResponse = serde_json::from_value(body).expect("assignments");
    assert_eq!(roles.role_ids, vec![everyone_id, first_role, second_role]);
    let (status, body) = api
        .get(&format!("{assignments}?offset=2&limit=1"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let roles: ListMemberRolesResponse = serde_json::from_value(body).expect("assignment page");
    assert_eq!(roles.role_ids, vec![second_role]);

    for member_id in [second.owner_membership_id, MemberId::new()] {
        assert_eq!(
            api.get(&format!("{base}/members/{member_id}/roles"), Some(&token))
                .await
                .0,
            StatusCode::NOT_FOUND
        );
    }
    for suffix in ["members", "ranks", "roles"] {
        let (status, body) = api
            .get(&format!("{base}/{suffix}?limit=1"), Some(&token))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body.get(suffix)
                .and_then(serde_json::Value::as_array)
                .expect("page items")
                .len(),
            1
        );
        let (status, body) = api
            .get(&format!("{base}/{suffix}?offset=100"), Some(&token))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.get(suffix)
                .and_then(serde_json::Value::as_array)
                .expect("empty page")
                .is_empty()
        );
    }
    for suffix in [
        "members".to_owned(),
        "ranks".to_owned(),
        "roles".to_owned(),
        format!("members/{}/roles", first.owner_membership_id),
    ] {
        for query in ["offset=-1", "limit=0", "limit=101"] {
            assert_eq!(
                api.get(&format!("{base}/{suffix}?{query}"), Some(&token))
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        assert_eq!(
            api.get(
                &format!("/api/v1/units/{}/{suffix}", UnitId::new()),
                Some(&token)
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }
    UnitMemberRoleStore::delete(&api.storage, first.owner_membership_id, first_role)
        .await
        .expect("remove first role");
    UnitMemberRoleStore::delete(&api.storage, first.owner_membership_id, second_role)
        .await
        .expect("remove second role");
    let (_, body) = api.get(&assignments, Some(&token)).await;
    let roles: ListMemberRolesResponse = serde_json::from_value(body).expect("no assignments");
    assert_eq!(roles.role_ids, vec![everyone_id]);
}

#[tokio::test(flavor = "multi_thread")]
async fn roster_reads_require_active_membership_in_the_requested_unit() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let first = unit(&api, owner, "first").await;
    let outsider = user(&api, "outsider").await;
    unit(&api, outsider, "other").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("owner token");
    let outsider_token = api
        .jwt
        .generate_jwt_for_user(outsider)
        .expect("outsider token");
    let missing_token = api
        .jwt
        .generate_jwt_for_user(UserId::new())
        .expect("missing user token");
    let base = format!("/api/v1/units/{}", first.unit.id);
    let paths = [
        format!("{base}/members"),
        format!("{base}/ranks"),
        format!("{base}/roles"),
        format!("{base}/members/{}/roles", first.owner_membership_id),
    ];
    for path in &paths {
        for token in [None, Some("invalid-token"), Some(missing_token.as_str())] {
            let (status, body) = api.get(path, token).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(
                body.get("code").and_then(serde_json::Value::as_str),
                Some("unauthorized")
            );
        }
        let (status, body) = api.get(path, Some(&outsider_token)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(
            body.get("code").and_then(serde_json::Value::as_str),
            Some("forbidden")
        );
    }
    let mut inactive = UserStore::get(&api.storage, owner)
        .await
        .expect("get owner")
        .expect("owner exists");
    inactive.is_active = false;
    UserStore::update(&api.storage, inactive)
        .await
        .expect("deactivate owner");
    for path in &paths {
        assert_eq!(
            api.get(path, Some(&token)).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    let mut active = UserStore::get(&api.storage, owner)
        .await
        .expect("get owner")
        .expect("owner exists");
    active.is_active = true;
    UserStore::update(&api.storage, active)
        .await
        .expect("reactivate owner");
    UnitMembershipStore::delete(&api.storage, first.owner_membership_id)
        .await
        .expect("remove membership");
    for path in &paths {
        assert_eq!(api.get(path, Some(&token)).await.0, StatusCode::FORBIDDEN);
    }
}
