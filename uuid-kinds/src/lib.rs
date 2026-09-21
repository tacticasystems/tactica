//! A common registry of UUIDs for use in Tactica.

mod ids;
pub use ids::*;

#[macro_export]
macro_rules! impl_typed_uuid_kinds {
    ($ident:ident) => {
        /// A typed UUID for $ident.
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        #[cfg_attr(
            feature = "diesel",
            derive(::diesel::AsExpression, ::diesel::FromSqlRow)
        )]
        #[cfg_attr(feature = "diesel", diesel(sql_type = ::diesel::sql_types::Uuid))]
        pub struct $ident(::uuid::Uuid);

        impl $ident {
            /// Creates a new instance of $ident with a random UUID.
            pub fn new() -> Self {
                $ident(::uuid::Uuid::now_v7())
            }

            /// Creates a new instance of $ident with a nil UUID.
            pub fn empty() -> Self {
                $ident(::uuid::Uuid::nil())
            }

            /// Returns the inner UUID.
            pub fn as_uuid(&self) -> &::uuid::Uuid {
                &self.0
            }

            #[cfg(feature = "chrono")]
            pub fn to_datetime(&self) -> ::chrono::DateTime<::chrono::Utc> {
                let timestamp: ::std::time::SystemTime = match self.0.get_timestamp() {
                    Some(ts) => ts,
                    None => unreachable!(
                        "UUID does not contain a timestamp. This should never happen with UUIDv7."
                    ),
                }
                .into();
                timestamp.into()
            }
        }

        impl From<::uuid::Uuid> for $ident {
            fn from(uuid: ::uuid::Uuid) -> Self {
                $ident(uuid)
            }
        }

        impl From<$ident> for ::uuid::Uuid {
            fn from(id: $ident) -> Self {
                id.0
            }
        }

        impl ::std::fmt::Display for $ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<&str> for $ident {
            type Error = ::uuid::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let uuid = ::uuid::Uuid::parse_str(value)?;
                Ok($ident(uuid))
            }
        }

        impl TryFrom<String> for $ident {
            type Error = ::uuid::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                TryFrom::try_from(value.as_str())
            }
        }

        #[cfg(feature = "diesel")]
        impl<B> FromSql<SqlUuid, B> for $ident
        where
            B: Backend,
            Uuid: FromSql<SqlUuid, B>,
        {
            fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> DeResult<Self> {
                <Uuid as FromSql<SqlUuid, B>>::from_sql(bytes).map($ident)
            }
        }

        #[cfg(feature = "diesel")]
        impl<B> ToSql<SqlUuid, B> for $ident
        where
            B: Backend,
            Uuid: ToSql<SqlUuid, B>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, B>) -> SerResult {
                <Uuid as ToSql<SqlUuid, B>>::to_sql(&self.0, out)
            }
        }

        #[cfg(feature = "utoipa")]
        impl utoipa::PartialSchema for $ident {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                        utoipa::openapi::schema::KnownFormat::Uuid,
                    )))
                    .build()
                    .into()
            }
        }
    };
}
