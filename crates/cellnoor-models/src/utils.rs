use std::{fmt::Debug, str::FromStr};

use diesel::{
    deserialize,
    deserialize::FromSql,
    pg::{Pg, PgValue},
    serialize::{self, ToSql},
    sql_types,
};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub trait UuidNewtypeFromSql: Sized + From<Uuid> {
    fn from_sql_inner(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let id = Uuid::from_sql(bytes)?;
        Ok(Self::from(id))
    }
}

pub trait UuidNewtypeToSql: AsRef<Uuid> {
    fn to_sql_inner<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        ToSql::<sql_types::Uuid, Pg>::to_sql(self.as_ref(), &mut out.reborrow())
    }
}

pub trait EnumFromSql: FromStr
where
    <Self as FromStr>::Err: Debug + std::error::Error + Send + Sync + 'static,
{
    fn from_sql_inner(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let string: String = FromSql::<sql_types::Text, Pg>::from_sql(bytes)?;
        Ok(Self::from_str(&string)?)
    }
}

pub trait EnumToSql
where
    &'static str: for<'a> From<&'a Self>,
{
    fn to_sql_inner<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        let as_str: &'static str = self.into();
        ToSql::<sql_types::Text, Pg>::to_sql(as_str, &mut out.reborrow())
    }
}

pub trait JsonFromSql: Sized + DeserializeOwned {
    fn from_sql_inner(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }

        Ok(serde_json::from_slice(&bytes[1..])?)
    }
}

#[allow(dead_code)]
pub trait JsonToSql: Serialize {
    fn to_sql_inner<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        use std::io::prelude::*;

        out.write_all(&[1])?;
        serde_json::to_writer(out, self)
            .map(|()| ::diesel::serialize::IsNull::No)
            .map_err(Into::into)
    }
}
