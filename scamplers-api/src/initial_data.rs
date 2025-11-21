use std::str::FromStr;

use diesel::{
    PgConnection, RunQueryDsl,
    prelude::*,
    sql_types::{Array, Text},
};
use scamplers_models::{
    institution,
    person::{self, UserRole},
};
use scamplers_schema::{institutions, people};
use uuid::Uuid;

use crate::validate::Validate;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InitialData {
    institution: institution::Creation,
    app_admin: person::Creation,
    // index_set_urls: Vec<Url>,
    // tenx_assays: Vec<NewTenxAssay>,
    // multiplexing_tags: Vec<NewMultiplexingTag>,
}

impl InitialData {
    pub fn institution(&self) -> &institution::Creation {
        &self.institution
    }

    pub fn app_admin(&self) -> &person::Creation {
        &self.app_admin
    }
}

pub async fn insert_initial_data(
    initial_data: InitialData,
    _http_client: reqwest::Client,
    db_pool: deadpool_diesel::postgres::Pool,
) -> anyhow::Result<()> {
    let simple_operations = |db_conn: &mut PgConnection| -> Result<(), anyhow::Error> {
        initial_data.validate(db_conn)?;

        let InitialData {
            institution,
            app_admin,
            // index_set_urls,
            // tenx_assays,
            // multiplexing_tags,
        } = initial_data;

        institution.upsert(db_conn)?;
        app_admin.upsert(db_conn)?;

        // // This is a loop of like 25 max
        // for assay in tenx_assays {
        //     duplicate_resource_ok(assay.execute(db_conn))?;
        // }

        // multiplexing_tags.execute(db_conn)?;

        Ok(())
    };

    // Insert index sets first so we can insert library-type specifications as part
    // of a larger set of operations in one shot
    // let db_conn = db_pool.get().await?;
    // download_and_insert_index_sets(&index_set_urls, http_client, db_conn).await?;

    let db_conn = db_pool.get().await?;
    db_conn.interact(simple_operations).await.unwrap()?;

    Ok(())
}

trait Upsert {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()>;
}

impl Upsert for institution::Creation {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        diesel::insert_into(institutions::table)
            .values(&self)
            .on_conflict(institutions::id)
            .do_update()
            .set(&self)
            .execute(db_conn)?;

        Ok(())
    }
}

impl Upsert for person::Creation {
    fn upsert(mut self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        define_sql_function! {fn create_user_if_not_exists(user_id: Text, password: Text, roles: Array<Text>)}

        diesel::update(people::table)
            .filter(people::email.eq(self.email()))
            .set(people::email.eq(None::<String>))
            .execute(db_conn)?;

        let id: Uuid = diesel::insert_into(people::table)
            .values(&self)
            .on_conflict(people::microsoft_entra_oid)
            .do_update()
            .set(&self)
            .returning(people::id)
            .get_result(db_conn)?;

        // Create a db user corresponding to this person so we can assign them a role.
        // Note that we set a random password so that nobody can log into the database
        // as that user.
        self.roles_mut().push(UserRole::AppAdmin);

        diesel::select(create_user_if_not_exists(
            id.to_string(),
            Uuid::now_v7().to_string(),
            self.roles(),
        ))
        .execute(db_conn)?;

        Ok(())
    }
}

impl FromStr for InitialData {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
