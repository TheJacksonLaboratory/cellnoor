#[macro_export]
macro_rules! impl_enum_from_sql {
    ($name:ident) => {
        #[cfg(feature = "app")]
        impl ::diesel::deserialize::FromSql<::diesel::sql_types::Text, ::diesel::pg::Pg> for $name {
            fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
                Self::from_sql_inner(bytes)
            }
        }

        #[cfg(feature = "app")]
        impl
            ::diesel::deserialize::FromSql<
                ::cellnoor_schema::sql_types::CaseInsensitiveText,
                ::diesel::pg::Pg,
            > for $name
        {
            fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
                Self::from_sql_inner(bytes)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_enum_to_sql {
    ($name:ident) => {
        #[cfg(feature = "app")]
        impl ::diesel::serialize::ToSql<::diesel::sql_types::Text, ::diesel::pg::Pg> for $name {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>,
            ) -> ::diesel::serialize::Result {
                self.to_sql_inner(out)
            }
        }

        #[cfg(feature = "app")]
        impl
            ::diesel::serialize::ToSql<
                ::cellnoor_schema::sql_types::CaseInsensitiveText,
                ::diesel::pg::Pg,
            > for $name
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>,
            ) -> ::diesel::serialize::Result {
                self.to_sql_inner(out)
            }
        }

        #[cfg(feature = "app")]
        impl
            ::diesel::serialize::ToSql<
                ::diesel::sql_types::Nullable<cellnoor_schema::sql_types::CaseInsensitiveText>,
                ::diesel::pg::Pg,
            > for $name
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>,
            ) -> ::diesel::serialize::Result {
                self.to_sql_inner(out)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_json_from_sql {
    ($name:ty) => {
        #[cfg(feature = "app")]
        impl ::diesel::deserialize::FromSql<::diesel::sql_types::Jsonb, ::diesel::pg::Pg>
            for $name
        {
            fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
                Self::from_sql_inner(bytes)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_json_to_sql {
    ($name:ty) => {
        #[cfg(feature = "app")]
        impl ::diesel::serialize::ToSql<::diesel::sql_types::Jsonb, ::diesel::pg::Pg> for $name {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>,
            ) -> ::diesel::serialize::Result {
                self.to_sql_inner(out)
            }
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! uuid_newtype {
    ($name:ident, $endpoint:literal) => {
        #[derive(Debug, Clone, Copy, ::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression, ::axum_extra::routing::TypedPath))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Uuid), typed_path($endpoint))]
        pub struct $name(pub ::uuid::Uuid);

        impl $name {
            pub fn to_id_string(&self) -> String {
                self.0.to_string()
            }
        }

        impl PartialEq<::uuid::Uuid> for $name {
            fn eq(&self, other: &::uuid::Uuid) -> bool {
                self.0 == *other
            }
        }

        impl AsRef<::uuid::Uuid> for $name {
            fn as_ref(&self) -> &::uuid::Uuid {
                &self.0
            }
        }

        impl From<::uuid::Uuid> for $name {
            fn from(value: ::uuid::Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for ::uuid::Uuid {
            fn from(val: $name) -> ::uuid::Uuid {
                val.0
            }
        }

        impl From<$name> for Vec<::uuid::Uuid> {
            fn from(val: $name) -> Vec<::uuid::Uuid> {
                vec![val.0]
            }
        }

        #[cfg(feature = "app")]
        impl crate::utils::UuidNewtypeFromSql for $name {}

        #[cfg(feature = "app")]
        impl crate::utils::UuidNewtypeToSql for $name {}

        #[cfg(feature = "app")]
        impl ::diesel::deserialize::FromSql<::diesel::sql_types::Uuid, ::diesel::pg::Pg> for $name {
            fn from_sql(bytes: ::diesel::pg::PgValue<'_>) -> ::diesel::deserialize::Result<Self> {
                use crate::utils::UuidNewtypeFromSql;
                Self::from_sql_inner(bytes)
            }
        }

        #[cfg(feature = "app")]
        impl ::diesel::serialize::ToSql<::diesel::sql_types::Uuid, ::diesel::pg::Pg> for $name {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>,
            ) -> ::diesel::serialize::Result {
                use crate::utils::UuidNewtypeToSql;
                self.to_sql_inner(out)
            }
        }

    };
}
