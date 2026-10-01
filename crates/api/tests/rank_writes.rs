mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use support::ApiFixture;
use tactica_api_types::v1::ranks::{ListRanksResponse, RankSummary};
use tactica_db_model::{
    CreateUnit, CreatedUnit, NewUnit, NewUnitMemberRole, NewUnitMembership, NewUnitRole, NewUser,
    UnitMemberRoleStore, UnitMembershipStore, UnitRoleStore, UnitStore, UserStore,
};
use tactica_uuid_kinds::{MemberId, RankId, RoleId, UnitId, UserId};
use tower::ServiceExt;

async fn request(
    api: &ApiFixture,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let response = api
        .router
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

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

async fn create_unit(api: &ApiFixture, owner_id: UserId, slug: &str) -> CreatedUnit {
    UnitStore::create_with_defaults(
        &api.storage,
        CreateUnit {
            owner_id,
            unit: NewUnit {
                id: UnitId::new(),
                slug: slug.to_owned(),
                display_name: None,
                icon_url: None,
                banner_url: None,
                biography: None,
            },
        },
    )
    .await
    .expect("create unit")
}

async fn create_rank(api: &ApiFixture, token: &str, unit_id: UnitId, slug: &str) -> RankSummary {
    let (status, body) = request(api, "POST", &format!("/api/v1/units/{unit_id}/ranks"), Some(token),
        json!({"slug": slug, "display_name": "Sergeant", "description": "Original", "icon_url": "https://example.test/icon.png"})).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    serde_json::from_value(body).expect("created rank")
}

#[tokio::test(flavor = "multi_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "Lifecycle and rejection checks share the same ranked unit fixture"
)]
async fn rank_crud_order_validation_and_in_use_protection() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let unit = create_unit(&api, owner, "unit").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("token");
    let base = format!("/api/v1/units/{}/ranks", unit.unit.id);
    let rank = create_rank(&api, &token, unit.unit.id, "  Sgt.  ").await;
    assert_eq!(rank.slug, "Sgt.");
    assert_eq!(rank.position, 0);
    let path = format!("{base}/{}", rank.id);
    let (status, body) = request(
        &api,
        "PATCH",
        &path,
        Some(&token),
        json!({"display_name": "  Staff Sergeant  "}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Staff Sergeant");
    assert_eq!(body["description"], "Original");
    assert_eq!(body["slug"], "Sgt.");
    let (status, body) = request(
        &api,
        "PATCH",
        &path,
        Some(&token),
        json!({"display_name": null, "description": null, "icon_url": null}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    for field in ["display_name", "description", "icon_url"] {
        assert!(body[field].is_null());
    }
    let (status, body) = api.get(&format!("{base}?limit=100"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let before: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    assert!(
        before
            .ranks
            .windows(2)
            .all(|pair| pair[0].position > pair[1].position)
    );
    let ids: Vec<_> = before.ranks.iter().map(|rank| rank.id).collect();
    // Sending display order as lowest-first deliberately reverses the order.
    let (status, body) = request(
        &api,
        "PATCH",
        &format!("{base}/order"),
        Some(&token),
        json!({"rank_ids": ids}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let after: ListRanksResponse = serde_json::from_value(body).expect("ordered ranks");
    assert_eq!(
        after.ranks.iter().map(|rank| rank.id).collect::<Vec<_>>(),
        ids.iter().rev().copied().collect::<Vec<_>>()
    );
    let (status, body) = api
        .get(&format!("{base}?offset=1&limit=1"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ranks"][0]["id"], json!(after.ranks[1].id));
    for bad in [
        vec![],
        vec![rank.id; ids.len()],
        vec![RankId::new(); ids.len()],
    ] {
        assert_eq!(
            request(
                &api,
                "PATCH",
                &format!("{base}/order"),
                Some(&token),
                json!({"rank_ids": bad})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(&api, "POST", &base, Some(&token), json!({"slug": "Sgt."}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    for body in [
        json!({"slug": " "}),
        json!({"slug": "a".repeat(101)}),
        json!({"slug": "new", "description": "x".repeat(2001)}),
        json!({"slug": "new", "icon_url": "javascript:alert(1)"}),
    ] {
        assert_eq!(
            request(&api, "POST", &base, Some(&token), body).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    assert!(
        request(
            &api,
            "POST",
            &base,
            Some(&token),
            json!({"slug": "new", "position": 10})
        )
        .await
        .0
        .is_client_error()
    );
    for body in [
        json!({"slug": " "}),
        json!({"position": 0}),
        json!({"unit_id": UnitId::new()}),
        json!({"id": RankId::new()}),
    ] {
        let status = request(&api, "PATCH", &path, Some(&token), body).await.0;
        assert!(status.is_client_error());
    }
    for existing in &before.ranks {
        if existing.id != rank.id {
            assert_eq!(
                request(
                    &api,
                    "DELETE",
                    &format!("{base}/{}", existing.id),
                    Some(&token),
                    json!(null)
                )
                .await
                .0,
                StatusCode::CONFLICT
            );
        }
    }
    assert_eq!(
        request(&api, "DELETE", &path, Some(&token), json!(null))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&api, "DELETE", &path, Some(&token), json!(null))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test(flavor = "multi_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "Exercise grants and revocation with the same membership and token"
)]
async fn ranks_visible_to_members_but_writes_require_live_role_permission() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let unit = create_unit(&api, owner, "unit").await;
    let owner_token = api.jwt.generate_jwt_for_user(owner).expect("token");
    let member = user(&api, "member").await;
    let membership = UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            unit_id: unit.unit.id,
            user_id: member,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("membership");
    let token = api.jwt.generate_jwt_for_user(member).expect("token");
    let outsider = user(&api, "outsider").await;
    let outsider_token = api.jwt.generate_jwt_for_user(outsider).expect("token");
    let base = format!("/api/v1/units/{}/ranks", unit.unit.id);
    assert_eq!(api.get(&base, Some(&token)).await.0, StatusCode::OK);
    assert_eq!(
        api.get(&base, Some(&outsider_token)).await.0,
        StatusCode::FORBIDDEN
    );
    let rank = create_rank(&api, &owner_token, unit.unit.id, "Sgt.").await;
    let (_, body) = api.get(&format!("{base}?limit=100"), Some(&token)).await;
    let ranks: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    let ids: Vec<_> = ranks.ranks.iter().rev().map(|rank| rank.id).collect();
    let writes = [
        ("POST", base.clone(), json!({"slug": "Cpl."})),
        (
            "PATCH",
            format!("{base}/{}", rank.id),
            json!({"description": "Changed"}),
        ),
        ("DELETE", format!("{base}/{}", rank.id), json!(null)),
        ("PATCH", format!("{base}/order"), json!({"rank_ids": ids})),
    ];
    for (method, path, body) in &writes {
        assert_eq!(
            request(&api, method, path, Some(&token), body.clone())
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(&api, method, path, Some(&outsider_token), body.clone())
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            request(&api, method, path, None, body.clone()).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    let mut role = UnitRoleStore::create(
        &api.storage,
        NewUnitRole {
            id: RoleId::new(),
            unit_id: unit.unit.id,
            display_name: "Rank manager".to_owned(),
            description: None,
            permissions: 16,
        },
    )
    .await
    .expect("role");
    UnitMemberRoleStore::assign(
        &api.storage,
        NewUnitMemberRole {
            unit_id: unit.unit.id,
            member_id: membership.id,
            role_id: role.id,
        },
    )
    .await
    .expect("assign role");
    let created = create_rank(&api, &token, unit.unit.id, "Cpl.").await;
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/{}", rank.id),
            Some(&token),
            json!({"description": "Changed"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, body) = api.get(&format!("{base}?limit=100"), Some(&token)).await;
    let ranks: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    let ids: Vec<_> = ranks.ranks.iter().map(|rank| rank.id).collect();
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/order"),
            Some(&token),
            json!({"rank_ids": ids})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &api,
            "DELETE",
            &format!("{base}/{}", created.id),
            Some(&token),
            json!(null)
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    // AssignRanks and other management bits cannot edit rank definitions.
    for mask in [32, 4, 8, 64, 0] {
        role.permissions = mask;
        role = UnitRoleStore::update(&api.storage, role)
            .await
            .expect("revoke rank permission");
        assert_eq!(
            request(
                &api,
                "PATCH",
                &format!("{base}/{}", rank.id),
                Some(&token),
                json!({"description": "Denied"})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    role.permissions = 1;
    role = UnitRoleStore::update(&api.storage, role)
        .await
        .expect("administrator");
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/{}", rank.id),
            Some(&token),
            json!({"description": "Administrator"})
        )
        .await
        .0,
        StatusCode::OK
    );
    // Cross-unit targets stay scoped even when both units are accessible.
    let other = create_unit(&api, owner, "other").await;
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/{}", other.owner_rank_id),
            Some(&owner_token),
            json!({"slug": "Foreign"})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &api,
            "DELETE",
            &format!("{base}/{}", other.owner_rank_id),
            Some(&owner_token),
            json!(null)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let mut account = UserStore::get(&api.storage, member)
        .await
        .expect("user")
        .expect("exists");
    account.is_active = false;
    UserStore::update(&api.storage, account)
        .await
        .expect("deactivate");
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/{}", rank.id),
            Some(&token),
            json!({})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(role.permissions, 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn everyone_can_grant_rank_management_and_concurrent_creates_preserve_order() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let created = create_unit(&api, owner, "unit").await;
    let member = user(&api, "member").await;
    UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            unit_id: created.unit.id,
            user_id: member,
            rank_id: created.owner_rank_id,
        },
    )
    .await
    .expect("membership");
    let token = api.jwt.generate_jwt_for_user(member).expect("token");
    let roles = UnitRoleStore::list(
        &api.storage,
        tactica_db_model::UnitRoleFilter::default().unit_id(vec![created.unit.id]),
        &tactica_db_model::ListPagination::unlimited(),
    )
    .await
    .expect("roles");
    let mut everyone = roles
        .into_iter()
        .find(|role| role.kind == "everyone")
        .expect("Everyone");
    everyone.permissions = 16;
    UnitRoleStore::update(&api.storage, everyone)
        .await
        .expect("grant Everyone");
    let (first, second) = tokio::join!(
        create_rank(&api, &token, created.unit.id, "A"),
        create_rank(&api, &token, created.unit.id, "B"),
    );
    assert_ne!(first.id, second.id);
    let base = format!("/api/v1/units/{}/ranks", created.unit.id);
    let (_, body) = api.get(&format!("{base}?limit=100"), Some(&token)).await;
    let before: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    assert_eq!(before.ranks.len(), 4);
    let positions: std::collections::HashSet<_> =
        before.ranks.iter().map(|rank| rank.position).collect();
    assert_eq!(positions.len(), before.ranks.len());
    let duplicate = request(&api, "POST", &base, Some(&token), json!({"slug": "A"})).await;
    assert_eq!(duplicate.0, StatusCode::CONFLICT);
    let (_, body) = api.get(&format!("{base}?limit=100"), Some(&token)).await;
    let after: ListRanksResponse = serde_json::from_value(body).expect("ranks");
    assert_eq!(
        before
            .ranks
            .iter()
            .map(|rank| (rank.id, rank.position))
            .collect::<Vec<_>>(),
        after
            .ranks
            .iter()
            .map(|rank| (rank.id, rank.position))
            .collect::<Vec<_>>()
    );
}
