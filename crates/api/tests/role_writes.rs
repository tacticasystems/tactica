mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use support::ApiFixture;
use tactica_api_types::v1::roles::RoleSummary;
use tactica_db_model::{
    CreateUnit, CreatedUnit, ListPagination, NewUnit, NewUnitMembership, NewUser,
    UnitMemberRoleFilter, UnitMemberRoleStore, UnitMembershipStore, UnitRoleStore, UnitStore,
    UserStore,
};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId, UserId};
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

async fn unit(api: &ApiFixture, owner_id: UserId, slug: &str) -> CreatedUnit {
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

async fn create_role(api: &ApiFixture, token: &str, unit_id: UnitId, name: &str) -> RoleSummary {
    let (status, body) = request(
        api,
        "POST",
        &format!("/api/v1/units/{unit_id}/roles"),
        Some(token),
        json!({"display_name": name, "description": "Original", "permissions": 7}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    serde_json::from_value(body).expect("created role")
}

#[tokio::test(flavor = "multi_thread")]
async fn owner_can_manage_roles_and_assignments_idempotently() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let unit = unit(&api, owner, "unit").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("token");
    let role = create_role(&api, &token, unit.unit.id, "  Medic  ").await;
    assert_eq!(role.display_name, "Medic");
    let path = format!("/api/v1/units/{}/roles/{}", unit.unit.id, role.id);
    let (status, body) = request(
        &api,
        "PATCH",
        &path,
        Some(&token),
        json!({"display_name": "Doctor"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: RoleSummary = serde_json::from_value(body).expect("updated role");
    assert_eq!(updated.display_name, "Doctor");
    assert_eq!(updated.description.as_deref(), Some("Original"));
    assert_eq!(updated.permissions, 7);
    let (status, body) = request(
        &api,
        "PATCH",
        &path,
        Some(&token),
        json!({"description": null, "permissions": 0}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: RoleSummary = serde_json::from_value(body).expect("cleared description");
    assert_eq!(updated.description, None);
    assert_eq!(updated.permissions, 0);
    assert_eq!(updated.display_name, "Doctor");
    let (status, body) = request(
        &api,
        "PATCH",
        &path,
        Some(&token),
        json!({"description": "New"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: RoleSummary = serde_json::from_value(body).expect("new description");
    assert_eq!(updated.description.as_deref(), Some("New"));

    let assignment = format!(
        "/api/v1/units/{}/members/{}/roles/{}",
        unit.unit.id, unit.owner_membership_id, role.id
    );
    let (first, second) = tokio::join!(
        request(&api, "PUT", &assignment, Some(&token), Value::Null),
        request(&api, "PUT", &assignment, Some(&token), Value::Null),
    );
    assert_eq!(first.0, StatusCode::NO_CONTENT);
    assert_eq!(second.0, StatusCode::NO_CONTENT);
    let assignments = UnitMemberRoleStore::list(
        &api.storage,
        UnitMemberRoleFilter::default().role_id(vec![role.id]),
        &ListPagination::unlimited(),
    )
    .await
    .expect("assignments");
    assert_eq!(assignments.len(), 1);
    for _ in 0..2 {
        assert_eq!(
            request(&api, "DELETE", &assignment, Some(&token), Value::Null)
                .await
                .0,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        request(&api, "PUT", &assignment, Some(&token), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&api, "DELETE", &path, Some(&token), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert!(
        UnitMemberRoleStore::list(
            &api.storage,
            UnitMemberRoleFilter::default().role_id(vec![role.id]),
            &ListPagination::unlimited()
        )
        .await
        .expect("cascaded assignments")
        .is_empty()
    );
    assert_eq!(
        request(&api, "DELETE", &path, Some(&token), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    // Authority survives deleting all ordinary roles and clearing their permissions.
    create_role(&api, &token, unit.unit.id, "Still owner").await;
    assert_eq!(
        api.get(
            &format!("/api/v1/units/{}/roles", unit.unit.id),
            Some(&token)
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn role_writes_validate_fields_conflicts_and_unit_scope() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let first = unit(&api, owner, "first").await;
    let second = unit(&api, owner, "second").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("token");
    let role = create_role(&api, &token, first.unit.id, "First").await;
    let other = create_role(&api, &token, second.unit.id, "Other").await;
    let base = format!("/api/v1/units/{}/roles", first.unit.id);
    for body in [
        json!({"display_name": " "}),
        json!({"display_name": "x".repeat(101)}),
        json!({"display_name": "Valid", "description": "x".repeat(2001)}),
        json!({"display_name": "Valid", "permissions": -1}),
    ] {
        assert_eq!(
            request(&api, "POST", &base, Some(&token), body.clone())
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(
                &api,
                "PATCH",
                &format!("{base}/{}", role.id),
                Some(&token),
                body
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &api,
            "POST",
            &base,
            Some(&token),
            json!({"display_name": " First "})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let conflict = create_role(&api, &token, first.unit.id, "Conflict").await;
    assert_eq!(
        request(
            &api,
            "PATCH",
            &format!("{base}/{}", conflict.id),
            Some(&token),
            json!({"display_name": "First"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    for id in [other.id, RoleId::new()] {
        assert_eq!(
            request(
                &api,
                "PATCH",
                &format!("{base}/{id}"),
                Some(&token),
                json!({"display_name": "Changed"})
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(
                &api,
                "DELETE",
                &format!("{base}/{id}"),
                Some(&token),
                Value::Null
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }
    for (member_id, role_id) in [
        (second.owner_membership_id, role.id),
        (first.owner_membership_id, other.id),
        (MemberId::new(), role.id),
        (first.owner_membership_id, RoleId::new()),
    ] {
        let path = format!(
            "/api/v1/units/{}/members/{member_id}/roles/{role_id}",
            first.unit.id
        );
        for method in ["PUT", "DELETE"] {
            assert_eq!(
                request(&api, method, &path, Some(&token), Value::Null)
                    .await
                    .0,
                StatusCode::NOT_FOUND
            );
        }
    }
    assert_eq!(
        UnitRoleStore::get(&api.storage, other.id)
            .await
            .expect("other role")
            .expect("exists")
            .display_name,
        "Other"
    );
    assert_eq!(
        request(
            &api,
            "POST",
            &base,
            Some(&token),
            json!({"display_name": "Bad", "unit_id": second.unit.id})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rank_and_superuser_status_do_not_grant_role_management() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let unit = unit(&api, owner, "unit").await;
    let token = api.jwt.generate_jwt_for_user(owner).expect("owner token");
    let role = create_role(&api, &token, unit.unit.id, "Role").await;
    let member = user(&api, "member").await;
    UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: member,
            unit_id: unit.unit.id,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("member with owner rank");
    let outsider = user(&api, "outsider").await;
    let mut superuser = UserStore::get(&api.storage, outsider)
        .await
        .expect("user")
        .expect("exists");
    superuser.is_superuser = true;
    UserStore::update(&api.storage, superuser)
        .await
        .expect("superuser");
    let member_token = api.jwt.generate_jwt_for_user(member).expect("member token");
    let outsider_token = api
        .jwt
        .generate_jwt_for_user(outsider)
        .expect("outsider token");
    let base = format!("/api/v1/units/{}/roles", unit.unit.id);
    let role_path = format!("{base}/{}", role.id);
    let assignment = format!(
        "/api/v1/units/{}/members/{}/roles/{}",
        unit.unit.id, unit.owner_membership_id, role.id
    );
    let routes = [
        ("POST", base, json!({"display_name": "Denied"})),
        (
            "PATCH",
            role_path.clone(),
            json!({"display_name": "Denied"}),
        ),
        ("DELETE", role_path, Value::Null),
        ("PUT", assignment.clone(), Value::Null),
        ("DELETE", assignment, Value::Null),
    ];
    for (method, path, body) in &routes {
        assert_eq!(
            request(&api, method, path, None, body.clone()).await.0,
            StatusCode::UNAUTHORIZED
        );
        for token in [&member_token, &outsider_token] {
            assert_eq!(
                request(&api, method, path, Some(token), body.clone())
                    .await
                    .0,
                StatusCode::FORBIDDEN
            );
        }
    }
    let mut user = UserStore::get(&api.storage, owner)
        .await
        .expect("owner")
        .expect("exists");
    user.is_active = false;
    UserStore::update(&api.storage, user)
        .await
        .expect("deactivate");
    for (method, path, body) in &routes {
        assert_eq!(
            request(&api, method, path, Some(&token), body.clone())
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        UnitRoleStore::get(&api.storage, role.id)
            .await
            .expect("role")
            .expect("exists")
            .display_name,
        "Role"
    );
}

struct HierarchyFixture {
    api: ApiFixture,
    unit_id: UnitId,
    owner_token: String,
    member_id: MemberId,
    member_token: String,
    // Lowest to highest: target, assign permission, manage permission,
    // administrator role, cosmetic ceiling, protected role.
    order: Vec<RoleId>,
    everyone: RoleId,
    builtin_admin: RoleId,
    target: RoleId,
    assign: RoleId,
    manage: RoleId,
    admin: RoleId,
    ceiling: RoleId,
    protected: RoleId,
}

impl HierarchyFixture {
    async fn new(grants: i64) -> Self {
        let api = ApiFixture::new().await;
        let owner_id = user(&api, "owner").await;
        let created = unit(&api, owner_id, "hierarchy").await;
        let owner_token = api
            .jwt
            .generate_jwt_for_user(owner_id)
            .expect("owner token");
        let member_user = user(&api, "delegate").await;
        let member_token = api
            .jwt
            .generate_jwt_for_user(member_user)
            .expect("delegate token");
        let member_id = MemberId::new();
        UnitMembershipStore::create(
            &api.storage,
            NewUnitMembership {
                id: member_id,
                user_id: member_user,
                unit_id: created.unit.id,
                rank_id: created.owner_rank_id,
            },
        )
        .await
        .expect("delegate membership");
        let (_, body) = request(
            &api,
            "GET",
            &format!("/api/v1/units/{}/roles", created.unit.id),
            Some(&owner_token),
            Value::Null,
        )
        .await;
        let defaults: tactica_api_types::v1::roles::ListRolesResponse =
            serde_json::from_value(body).expect("built-in roles");
        let everyone = defaults
            .roles
            .iter()
            .find(|role| role.kind == "everyone")
            .expect("Everyone")
            .id;
        let builtin_admin = defaults
            .roles
            .iter()
            .find(|role| role.kind == "administrator")
            .expect("Administrator")
            .id;
        let mut ids = Vec::new();
        for (name, mask) in [
            ("Protected", 0),
            ("Cosmetic ceiling", 0),
            ("Custom administrator", 1),
            ("Manage", 4),
            ("Assign", 8),
            ("Target", 0),
        ] {
            let (status, body) = request(
                &api,
                "POST",
                &format!("/api/v1/units/{}/roles", created.unit.id),
                Some(&owner_token),
                json!({"display_name": name, "permissions": mask}),
            )
            .await;
            assert_eq!(status, StatusCode::CREATED);
            ids.push(
                serde_json::from_value::<RoleSummary>(body)
                    .expect("role")
                    .id,
            );
        }
        let mut ids = ids.into_iter();
        let protected = ids.next().expect("protected");
        let ceiling = ids.next().expect("ceiling");
        let admin = ids.next().expect("admin");
        let manage = ids.next().expect("manage");
        let assign = ids.next().expect("assign");
        let target = ids.next().expect("target");
        let fixture = Self {
            api,
            unit_id: created.unit.id,
            owner_token,
            member_id,
            member_token,
            order: vec![
                everyone,
                target,
                assign,
                manage,
                admin,
                ceiling,
                protected,
                builtin_admin,
            ],
            everyone,
            builtin_admin,
            target,
            assign,
            manage,
            admin,
            ceiling,
            protected,
        };
        fixture.grant(ceiling).await;
        for (mask, role) in [(1, admin), (4, manage), (8, assign)] {
            if grants & mask != 0 {
                fixture.grant(role).await;
            }
        }
        fixture
    }

    fn roles_path(&self) -> String {
        format!("/api/v1/units/{}/roles", self.unit_id)
    }
    fn role_path(&self, role: RoleId) -> String {
        format!("{}/{}", self.roles_path(), role)
    }
    fn assignment_path(&self, role: RoleId) -> String {
        format!(
            "/api/v1/units/{}/members/{}/roles/{role}",
            self.unit_id, self.member_id
        )
    }
    async fn grant(&self, role: RoleId) {
        assert_eq!(
            request(
                &self.api,
                "PUT",
                &self.assignment_path(role),
                Some(&self.owner_token),
                Value::Null
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
    }
    async fn as_member(&self, method: &str, path: &str, body: Value) -> StatusCode {
        request(&self.api, method, path, Some(&self.member_token), body)
            .await
            .0
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn role_hierarchy_uses_highest_assigned_role_independently_of_its_permissions() {
    let f = HierarchyFixture::new(4 | 8).await;
    // Permission bits are combined across roles, including a zero-permission ceiling.
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"display_name": "Edited"})
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member("DELETE", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    for role in [f.ceiling, f.protected] {
        assert_eq!(
            f.as_member(
                "PATCH",
                &f.role_path(role),
                json!({"display_name": "Denied"})
            )
            .await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            f.as_member("DELETE", &f.role_path(role), Value::Null).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            f.as_member("PUT", &f.assignment_path(role), Value::Null)
                .await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            f.as_member("DELETE", &f.assignment_path(role), Value::Null)
                .await,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        f.as_member(
            "POST",
            &f.roles_path(),
            json!({"display_name": "Delegated", "permissions": 4 | 8})
        )
        .await,
        StatusCode::CREATED
    );
    for mask in [1, 2, 16, 32, 64] {
        assert_eq!(
            f.as_member(
                "POST",
                &f.roles_path(),
                json!({"display_name": "Denied", "permissions": mask})
            )
            .await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            f.as_member(
                "PATCH",
                &f.role_path(f.target),
                json!({"permissions": mask})
            )
            .await,
            StatusCode::FORBIDDEN
        );
    }
    // Existing privileged roles below the ceiling can be edited without granting
    // new bits; assignment is governed by position, not by a permission subset.
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.admin),
            json!({"display_name": "Renamed admin"})
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.admin), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member(
            "POST",
            &f.roles_path(),
            json!({"display_name": "Now administrator", "permissions": 127})
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.ceiling),
            json!({"display_name": "Still denied"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn manage_roles_and_assign_roles_are_separate_and_revocation_is_immediate() {
    let f = HierarchyFixture::new(4).await;
    assert_eq!(
        f.as_member("POST", &f.roles_path(), json!({"display_name": "Allowed"}))
            .await,
        StatusCode::CREATED
    );
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.as_member("DELETE", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::FORBIDDEN
    );
    f.grant(f.assign).await;
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &f.api,
            "DELETE",
            &f.assignment_path(f.manage),
            Some(&f.owner_token),
            Value::Null
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member("POST", &f.roles_path(), json!({"display_name": "Revoked"}))
            .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"description": "Denied"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.as_member("DELETE", &f.role_path(f.target), Value::Null)
            .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.as_member("DELETE", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn administrator_obeys_hierarchy_and_owner_can_reorder_every_role() {
    let f = HierarchyFixture::new(1).await;
    assert_eq!(
        f.as_member("PATCH", &f.role_path(f.target), json!({"permissions": 127}))
            .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.protected),
            json!({"description": "Denied"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    let order_path = format!("{}/order", f.roles_path());
    let allowed = vec![
        f.everyone,
        f.assign,
        f.target,
        f.manage,
        f.admin,
        f.ceiling,
        f.protected,
        f.builtin_admin,
    ];
    assert_eq!(
        f.as_member("PATCH", &order_path, json!({"role_ids": allowed}))
            .await,
        StatusCode::OK
    );
    let denied = vec![
        f.everyone,
        f.assign,
        f.manage,
        f.admin,
        f.ceiling,
        f.target,
        f.protected,
        f.builtin_admin,
    ];
    assert_eq!(
        f.as_member("PATCH", &order_path, json!({"role_ids": denied}))
            .await,
        StatusCode::FORBIDDEN
    );
    let (_, body) = request(
        &f.api,
        "GET",
        &f.roles_path(),
        Some(&f.member_token),
        Value::Null,
    )
    .await;
    let roles: tactica_api_types::v1::roles::ListRolesResponse =
        serde_json::from_value(body).expect("role listing");
    assert_eq!(
        roles.roles.iter().map(|role| role.id).collect::<Vec<_>>(),
        allowed.iter().rev().copied().collect::<Vec<_>>()
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &order_path,
            Some(&f.owner_token),
            json!({"role_ids": denied})
        )
        .await
        .0,
        StatusCode::OK
    );
    // The moved target is assigned to the actor, so its higher position becomes the ceiling.
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.ceiling),
            json!({"description": "Now below"})
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"description": "Own ceiling"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    for invalid in [
        vec![],
        vec![f.target; f.order.len()],
        vec![RoleId::new(); f.order.len()],
    ] {
        assert_eq!(
            request(
                &f.api,
                "PATCH",
                &order_path,
                Some(&f.owner_token),
                json!({"role_ids": invalid})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    for mask in [-1, 128, 129, i64::MAX] {
        assert_eq!(
            request(
                &f.api,
                "PATCH",
                &f.role_path(f.target),
                Some(&f.owner_token),
                json!({"permissions": mask})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn permissions_do_not_leak_across_units_or_stop_at_a_listing_page() {
    let f = HierarchyFixture::new(0).await;
    assert_eq!(
        f.as_member("POST", &f.roles_path(), json!({"display_name": "Denied"}))
            .await,
        StatusCode::FORBIDDEN
    );
    // More than the normal list page size, with the useful role assigned last.
    for index in 0..12 {
        let (status, body) = request(
            &f.api,
            "POST",
            &f.roles_path(),
            Some(&f.owner_token),
            json!({"display_name": format!("Extra {index}"), "permissions": 0}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let role: RoleSummary = serde_json::from_value(body).expect("role");
        f.grant(role.id).await;
    }
    f.grant(f.manage).await;
    assert_eq!(
        f.as_member(
            "POST",
            &f.roles_path(),
            json!({"display_name": "All roles counted"})
        )
        .await,
        StatusCode::CREATED
    );
    let member = UnitMembershipStore::get(&f.api.storage, f.member_id)
        .await
        .expect("member")
        .expect("exists");
    let other_owner = user(&f.api, "other-owner").await;
    let other = unit(&f.api, other_owner, "other").await;
    UnitMembershipStore::create(
        &f.api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: member.user_id,
            unit_id: other.unit.id,
            rank_id: other.owner_rank_id,
        },
    )
    .await
    .expect("membership in other unit");
    assert_eq!(
        f.as_member(
            "POST",
            &format!("/api/v1/units/{}/roles", other.unit.id),
            json!({"display_name": "No cross-unit permissions"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn builtin_roles_are_immutable_except_everyone_permissions_and_stay_pinned() {
    let f = HierarchyFixture::new(0).await;
    for body in [
        json!({}),
        json!({"display_name": "Renamed"}),
        json!({"description": "Changed"}),
        json!({"permissions": 0}),
    ] {
        assert_eq!(
            request(
                &f.api,
                "PATCH",
                &f.role_path(f.builtin_admin),
                Some(&f.owner_token),
                body
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    for body in [
        json!({"display_name": "Members"}),
        json!({"description": "Changed"}),
        json!({"description": null}),
    ] {
        assert_eq!(
            request(
                &f.api,
                "PATCH",
                &f.role_path(f.everyone),
                Some(&f.owner_token),
                body
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    for role in [f.everyone, f.builtin_admin] {
        assert_eq!(
            request(
                &f.api,
                "DELETE",
                &f.role_path(role),
                Some(&f.owner_token),
                Value::Null
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    for method in ["PUT", "DELETE"] {
        assert_eq!(
            request(
                &f.api,
                method,
                &f.assignment_path(f.everyone),
                Some(&f.owner_token),
                Value::Null
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let order_path = format!("{}/order", f.roles_path());
    let mut everyone_moved = f.order.clone();
    everyone_moved.rotate_left(1);
    let mut admin_moved = f.order.clone();
    admin_moved.rotate_right(1);
    for order in [everyone_moved, admin_moved] {
        assert_eq!(
            request(
                &f.api,
                "PATCH",
                &order_path,
                Some(&f.owner_token),
                json!({"role_ids": order})
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let (_, body) = request(
        &f.api,
        "GET",
        &f.roles_path(),
        Some(&f.owner_token),
        Value::Null,
    )
    .await;
    let roles: tactica_api_types::v1::roles::ListRolesResponse =
        serde_json::from_value(body).expect("roles");
    let admin = roles.roles.first().expect("highest");
    assert_eq!(admin.id, f.builtin_admin);
    assert_eq!(admin.display_name, "Administrator");
    assert_eq!(admin.permissions, 1);
    assert_eq!(admin.description, None);
    let everyone = roles.roles.last().expect("lowest");
    assert_eq!(everyone.id, f.everyone);
    assert_eq!(everyone.position, 0);
    assert_eq!(everyone.permissions, 0);
    assert_eq!(everyone.display_name, "Everyone");
    assert_eq!(everyone.description, None);
    assert_eq!(
        request(
            &f.api,
            "POST",
            &f.roles_path(),
            Some(&f.owner_token),
            json!({"display_name": "Spoof", "kind": "administrator"})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn everyone_is_implicit_for_current_and_future_members_and_changes_apply_immediately() {
    let f = HierarchyFixture::new(0).await;
    let newcomer_id = user(&f.api, "newcomer").await;
    let rank_id = UnitMembershipStore::get(&f.api.storage, f.member_id)
        .await
        .expect("member")
        .expect("exists")
        .rank_id;
    let newcomer = UnitMembershipStore::create(
        &f.api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: newcomer_id,
            unit_id: f.unit_id,
            rank_id,
        },
    )
    .await
    .expect("new member");
    let path = format!("/api/v1/units/{}/members/{}/roles", f.unit_id, newcomer.id);
    let (status, body) = request(&f.api, "GET", &path, Some(&f.owner_token), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let roles: tactica_api_types::v1::members::ListMemberRolesResponse =
        serde_json::from_value(body).expect("member roles");
    assert_eq!(roles.role_ids, vec![f.everyone]);
    assert!(
        UnitMemberRoleStore::list(
            &f.api.storage,
            UnitMemberRoleFilter::default().member_id(vec![newcomer.id]),
            &ListPagination::unlimited()
        )
        .await
        .expect("no explicit assignments")
        .is_empty()
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"description": "Denied"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &f.role_path(f.everyone),
            Some(&f.owner_token),
            json!({"permissions": 4 | 8})
        )
        .await
        .0,
        StatusCode::OK
    );
    // The delegate has only a cosmetic explicit role, and gets capabilities from Everyone.
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"description": "From Everyone"})
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member("PUT", &f.assignment_path(f.target), Value::Null)
            .await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.everyone),
            json!({"permissions": 1 | 4 | 8})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    let newcomer_token = f.api.jwt.generate_jwt_for_user(newcomer_id).expect("token");
    // Everyone alone is the lowest role; it cannot create a role above itself.
    assert_eq!(
        request(
            &f.api,
            "POST",
            &f.roles_path(),
            Some(&newcomer_token),
            json!({"display_name": "Above self"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let new_assignment = format!(
        "/api/v1/units/{}/members/{}/roles/{}",
        f.unit_id, newcomer.id, f.ceiling
    );
    assert_eq!(
        request(
            &f.api,
            "PUT",
            &new_assignment,
            Some(&f.owner_token),
            Value::Null
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &f.role_path(f.target),
            Some(&newcomer_token),
            json!({"description": "Inherited"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &f.role_path(f.everyone),
            Some(&f.owner_token),
            json!({"permissions": 0})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.target),
            json!({"description": "Revoked"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &f.role_path(f.target),
            Some(&newcomer_token),
            json!({"description": "Revoked"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    UnitMembershipStore::delete(&f.api.storage, newcomer.id)
        .await
        .expect("remove membership");
    assert!(
        UnitRoleStore::list_for_member(
            &f.api.storage,
            f.unit_id,
            newcomer.id,
            &ListPagination::unlimited()
        )
        .await
        .expect("former member roles")
        .is_empty()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn builtin_administrator_is_assignable_and_does_not_define_ownership() {
    let f = HierarchyFixture::new(0).await;
    let unit = UnitStore::get(&f.api.storage, f.unit_id)
        .await
        .expect("unit")
        .expect("exists");
    let owner_member =
        UnitMembershipStore::get_by_user_and_unit(&f.api.storage, unit.owner_id, f.unit_id)
            .await
            .expect("membership")
            .expect("owner member");
    let roles = UnitRoleStore::list_for_member(
        &f.api.storage,
        f.unit_id,
        owner_member.id,
        &ListPagination::unlimited(),
    )
    .await
    .expect("owner roles");
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.first().expect("Everyone").id, f.everyone);
    f.grant(f.builtin_admin).await;
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.protected),
            json!({"permissions": 127})
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.builtin_admin),
            json!({"permissions": 0})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.as_member("DELETE", &f.assignment_path(f.builtin_admin), Value::Null)
            .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &f.api,
            "DELETE",
            &f.assignment_path(f.builtin_admin),
            Some(&f.owner_token),
            Value::Null
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.as_member(
            "PATCH",
            &f.role_path(f.protected),
            json!({"description": "No longer admin"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        UnitStore::get(&f.api.storage, f.unit_id)
            .await
            .expect("unit")
            .expect("exists")
            .owner_id,
        unit.owner_id
    );
    assert_eq!(
        request(
            &f.api,
            "PATCH",
            &f.role_path(f.protected),
            Some(&f.owner_token),
            json!({"description": "Owner still controls unit"})
        )
        .await
        .0,
        StatusCode::OK
    );
}
