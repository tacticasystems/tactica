use crate::impl_typed_uuid_kinds;
#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    deserialize::{FromSql, Result as DeResult},
    serialize::{Output, Result as SerResult, ToSql},
    sql_types::Uuid as SqlUuid,
};
use uuid::Uuid;

impl_typed_uuid_kinds!(UserId);
impl_typed_uuid_kinds!(SessionId);
impl_typed_uuid_kinds!(UnitId);
impl_typed_uuid_kinds!(RankId);
impl_typed_uuid_kinds!(RoleId);
impl_typed_uuid_kinds!(MemberId);
impl_typed_uuid_kinds!(TeamId);
impl_typed_uuid_kinds!(SlotId);
