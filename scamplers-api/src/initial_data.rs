use std::str::FromStr;

use diesel::PgConnection;
use scamplers_models::{
    institution::InstitutionCreation, person::PersonCreation, tenx_assay::TenxAssayCreation,
};
use url::Url;

use crate::{initial_data::index_sets::download_and_insert_index_sets, validate::Validate};

mod app_admin;
mod index_sets;
mod institution;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InitialData {
    institution: InstitutionCreation,
    app_admin: PersonCreation,
    index_set_urls: Vec<Url>,
    tenx_assays: Vec<TenxAssayCreation>,
    // multiplexing_tags: Vec<NewMultiplexingTag>,
}

impl InitialData {
    pub fn institution(&self) -> &InstitutionCreation {
        &self.institution
    }

    pub fn app_admin(&self) -> &PersonCreation {
        &self.app_admin
    }

    pub fn index_set_urls(&self) -> &[Url] {
        &self.index_set_urls
    }

    pub fn tenx_assays(&self) -> &[TenxAssayCreation] {
        &self.tenx_assays
    }
}

pub async fn insert_initial_data(
    initial_data: InitialData,
    http_client: reqwest::Client,
    db_pool: deadpool_diesel::postgres::Pool,
) -> anyhow::Result<()> {
    let db_conn = db_pool.get().await?;

    let initial_data = db_conn
        .interact(move |db_conn| initial_data.validate(db_conn).map(|()| initial_data))
        .await
        .unwrap()?;

    let InitialData {
        institution,
        app_admin,
        index_set_urls,
        tenx_assays: _,
    } = initial_data;

    let simple_operations = |db_conn: &mut PgConnection| -> Result<(), anyhow::Error> {
        institution.upsert(db_conn)?;
        app_admin.upsert(db_conn)?;

        // // This is a loop of like 25 max
        // for assay in tenx_assays {
        //     duplicate_resource_ok(assay.execute(db_conn))?;
        // }

        // multiplexing_tags.execute(db_conn)?;

        Ok(())
    };

    let db_conn = db_pool.get().await?;
    download_and_insert_index_sets(index_set_urls, http_client, db_conn).await?;

    let db_conn = db_pool.get().await?;
    db_conn.interact(simple_operations).await.unwrap()?;

    Ok(())
}

trait Upsert {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()>;
}

impl FromStr for InitialData {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
