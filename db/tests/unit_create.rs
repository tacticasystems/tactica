mod support;

use tactica_db_model::{
    CreateUnit, ListPagination, NewUnit, NewUser, UnitFilter, UnitMembershipFilter,
    UnitMembershipStore, UnitRankFilter, UnitRankStore, UnitSettingsStore, UnitStore, UserStore,
};
use tactica_uuid_kinds::{UnitId, UserId};

#[tokio::test(flavor = "multi_thread")]
async fn create_with_defaults_commits_unit_and_defaults() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;
    let owner_id = UserId::new();
    UserStore::create(
        &storage,
        NewUser {
            id: owner_id,
            username: "unit-owner".to_owned(),
            email: "owner@example.test".to_owned(),
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
    .expect("create owner user");

    let created = storage
        .create_with_defaults(CreateUnit {
            owner_id,
            unit: NewUnit {
                id: UnitId::new(),
                slug: "atomic-create".to_owned(),
                display_name: Some("Atomic Create".to_owned()),
                icon_url: None,
                banner_url: None,
                biography: None,
            },
        })
        .await
        .expect("create unit and defaults");

    assert_eq!(created.unit.slug, "atomic-create");
    let units = UnitStore::list(&storage, UnitFilter::default(), &ListPagination::default())
        .await
        .expect("list units");
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].id, created.unit.id);

    let ranks = UnitRankStore::list(
        &storage,
        UnitRankFilter::default().unit_id(vec![created.unit.id]),
        &ListPagination::unlimited(),
    )
    .await
    .expect("list default ranks");
    assert_eq!(ranks.len(), 2);
    assert!(ranks.iter().any(|rank| rank.id == created.owner_rank_id));

    let settings = UnitSettingsStore::get(&storage, created.unit.id)
        .await
        .expect("get unit settings")
        .expect("default settings exist");
    assert!(ranks.iter().any(|rank| rank.id == settings.initial_rank_id));

    let memberships = UnitMembershipStore::list(
        &storage,
        UnitMembershipFilter::default().unit_id(vec![created.unit.id]),
        &ListPagination::default(),
    )
    .await
    .expect("list owner membership");
    assert_eq!(memberships.len(), 1);
    assert_eq!(memberships[0].id, created.owner_membership_id);
    assert_eq!(memberships[0].rank_id, created.owner_rank_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_with_defaults_rolls_back_when_a_later_insert_fails() {
    let database = support::DatabaseFixture::new().await;
    let storage = database.connect().await;

    let result = storage
        .create_with_defaults(CreateUnit {
            // This user does not exist, so inserting settings fails after the
            // unit and both ranks have already been inserted in the transaction.
            owner_id: UserId::new(),
            unit: NewUnit {
                id: UnitId::new(),
                slug: "must-roll-back".to_owned(),
                display_name: Some("Must Roll Back".to_owned()),
                icon_url: None,
                banner_url: None,
                biography: None,
            },
        })
        .await;
    assert!(result.is_err());

    let units = UnitStore::list(&storage, UnitFilter::default(), &ListPagination::default())
        .await
        .expect("list units after rollback");
    assert!(units.is_empty(), "the unit insert must be rolled back");
}
