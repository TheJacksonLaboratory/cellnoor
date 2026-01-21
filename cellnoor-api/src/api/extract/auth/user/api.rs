use cellnoor_schema::{lab_membership, labs, people};
use deadpool_diesel::postgres::Pool;
use diesel::{HasQuery, PgConnection, prelude::*};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{self, DbConnection};

use super::{FromEncodedJwt, common::*};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct User {
    standard_claims: StandardClaims,
    user: PrivateClaims,
}

impl User {
    pub fn id(&self) -> Uuid {
        self.standard_claims.sub
    }

    pub(super) async fn from_standard_claims(
        standard_claims: StandardClaims,
        db_conn: DbConnection,
    ) -> Result<Self, db::Error> {
        let user_id = standard_claims.sub;

        let user_fields = db_conn.interact(move |db_conn| {
            UserFields::query()
                .filter(people::id.eq(user_id))
                .first(db_conn)
        });

        let user_labs = db_conn.interact(move |db_conn| {
            lab_membership::table
                .select(lab_membership::lab_id)
                .filter(lab_membership::member_id.eq(user_id))
                .load(db_conn)
        });

        let (user_fields, user_labs) = tokio::try_join!(user_fields, user_labs)?;

        Ok(Self {
            standard_claims,
            user: PrivateClaims {
                user_fields: user_fields?,
                labs: user_labs?,
            },
        })
    }
}
