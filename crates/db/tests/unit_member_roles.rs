mod support;

use diesel::result::{DatabaseErrorKind, Error};
use tactica_db::PgConnection;
use tactica_db_model::{
    CreateUnit, CreatedUnit, ListPagination, NewUnit, NewUnitMemberRole, NewUnitMembership,
    NewUnitRole, NewUser, StoreError, UnitMemberRole, UnitMemberRoleFilter, UnitMemberRoleStore,
    UnitMembershipStore, UnitRoleStore, UnitStore, UserStore,
};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId, UserId};

async fn user(storage: &PgConnection, name: &str) -> UserId {
    let id = UserId::new();
    UserStore::create(
        storage,
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

async fn unit(storage: &PgConnection, owner_id: UserId, slug: &str) -> CreatedUnit {
    storage
        .create_with_defaults(CreateUnit {
            owner_id,
            unit: NewUnit {
                id: UnitId::new(),
                slug: slug.to_owned(),
                display_name: None,
                icon_url: None,
                banner_url: None,
                biography: None,
            },
        })
        .await
        .expect("create unit")
}

async fn role(storage: &PgConnection, unit_id: UnitId, name: &str) -> RoleId {
    let id = RoleId::new();
    UnitRoleStore::create(
        storage,
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
    id
}

async fn assign(
    storage: &PgConnection,
    unit_id: UnitId,
    member_id: MemberId,
    role_id: RoleId,
) -> Result<UnitMemberRole, StoreError> {
    UnitMemberRoleStore::create(
        storage,
        NewUnitMemberRole {
            unit_id,
            member_id,
            role_id,
        },
    )
    .await
}

async fn list(storage: &PgConnection, filter: UnitMemberRoleFilter) -> Vec<UnitMemberRole> {
    UnitMemberRoleStore::list(storage, filter, &ListPagination::unlimited())
        .await
        .expect("list assignments")
}

async fn assert_assignment_pagination(storage: &PgConnection) {
    let all = list(storage, UnitMemberRoleFilter::default()).await;
    let page = UnitMemberRoleStore::list(
        storage,
        UnitMemberRoleFilter::default(),
        &ListPagination::default().offset(1).limit(1),
    )
    .await
    .expect("page assignments");
    assert_eq!(page.len(), 1);
    assert_eq!(
        page.first().expect("one assignment").member_id,
        all.get(1).expect("second assignment").member_id
    );
    assert_eq!(
        page.first().expect("one assignment").role_id,
        all.get(1).expect("second assignment").role_id
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn assignments_are_many_to_many_filterable_and_removable() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let owner = user(&storage, "owner").await;
    let unit = unit(&storage, owner, "test-unit").await;
    let member = UnitMembershipStore::create(
        &storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: user(&storage, "member").await,
            unit_id: unit.unit.id,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("create member");
    let first = role(&storage, unit.unit.id, "First").await;
    let second = role(&storage, unit.unit.id, "Second").await;
    for (member_id, role_id) in [
        (unit.owner_membership_id, first),
        (unit.owner_membership_id, second),
        (member.id, first),
    ] {
        let assignment = assign(&storage, unit.unit.id, member_id, role_id)
            .await
            .expect("assign role");
        assert_eq!(assignment.member_id, member_id);
        assert_eq!(assignment.role_id, role_id);
        assert_eq!(assignment.unit_id, unit.unit.id);
    }
    assert!(matches!(
        assign(&storage, unit.unit.id, member.id, first).await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        list(
            &storage,
            UnitMemberRoleFilter::default().member_id(vec![unit.owner_membership_id])
        )
        .await
        .len(),
        2
    );
    assert_eq!(
        list(
            &storage,
            UnitMemberRoleFilter::default().role_id(vec![first])
        )
        .await
        .len(),
        2
    );
    assert_eq!(
        list(
            &storage,
            UnitMemberRoleFilter::default().unit_id(vec![unit.unit.id])
        )
        .await
        .len(),
        3
    );
    assert!(
        list(
            &storage,
            UnitMemberRoleFilter::default().unit_id(vec![UnitId::new()])
        )
        .await
        .is_empty()
    );
    assert!(
        list(&storage, UnitMemberRoleFilter::default().role_id(vec![]))
            .await
            .is_empty()
    );
    let filter = UnitMemberRoleFilter::default()
        .member_id(vec![member.id])
        .role_id(vec![second]);
    assert!(list(&storage, filter).await.is_empty());
    assert_assignment_pagination(&storage).await;
    for _ in 0..2 {
        UnitMemberRoleStore::delete(&storage, unit.owner_membership_id, first)
            .await
            .expect("remove assignment idempotently");
    }
    assert_eq!(
        list(&storage, UnitMemberRoleFilter::default()).await.len(),
        2
    );
    assert_eq!(
        list(
            &storage,
            UnitMemberRoleFilter::default()
                .member_id(vec![member.id])
                .role_id(vec![first])
        )
        .await
        .len(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn assignments_require_existing_members_and_roles_in_the_same_unit() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let owner = user(&storage, "owner").await;
    let first = unit(&storage, owner, "first").await;
    let second = unit(&storage, owner, "second").await;
    let role_id = role(&storage, first.unit.id, "Role").await;
    for (unit_id, member_id, role_id) in [
        (first.unit.id, second.owner_membership_id, role_id),
        (second.unit.id, second.owner_membership_id, role_id),
        (first.unit.id, MemberId::new(), role_id),
        (first.unit.id, first.owner_membership_id, RoleId::new()),
        (UnitId::new(), first.owner_membership_id, role_id),
    ] {
        assert!(matches!(
            assign(&storage, unit_id, member_id, role_id).await,
            Err(StoreError::Db(Error::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                _
            )))
        ));
    }
    assert!(
        list(&storage, UnitMemberRoleFilter::default())
            .await
            .is_empty()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deleting_a_role_member_or_unit_removes_assignments() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let owner = user(&storage, "owner").await;
    let first = unit(&storage, owner, "first").await;
    let second = unit(&storage, owner, "second").await;
    let first_role = role(&storage, first.unit.id, "First").await;
    let second_role = role(&storage, first.unit.id, "Second").await;
    let other_role = role(&storage, second.unit.id, "Other").await;
    for (unit_id, member_id, role_id) in [
        (first.unit.id, first.owner_membership_id, first_role),
        (first.unit.id, first.owner_membership_id, second_role),
        (second.unit.id, second.owner_membership_id, other_role),
    ] {
        assign(&storage, unit_id, member_id, role_id)
            .await
            .expect("assign role");
    }
    UnitRoleStore::delete(&storage, first_role)
        .await
        .expect("delete role");
    assert_eq!(
        list(&storage, UnitMemberRoleFilter::default()).await.len(),
        2
    );
    UnitMembershipStore::delete(&storage, first.owner_membership_id)
        .await
        .expect("delete member");
    let remaining = list(&storage, UnitMemberRoleFilter::default()).await;
    assert_eq!(remaining.len(), 1);
    assert_eq!(
        remaining.first().expect("remaining assignment").role_id,
        other_role
    );
    UnitStore::delete(&storage, second.unit.id)
        .await
        .expect("delete unit");
    assert!(
        list(&storage, UnitMemberRoleFilter::default())
            .await
            .is_empty()
    );
}
