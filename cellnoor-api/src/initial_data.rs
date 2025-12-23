use std::str::FromStr;

use cellnoor_models::{
    institution::InstitutionCreation, multiplexing_tag::MultiplexingTagCreation,
    person::PersonCreation, tenx_assay::TenxAssayCreation,
};
use diesel::PgConnection;
pub(crate) use index_sets::IndexSetName;
use url::Url;

use crate::{
    initial_data::index_sets::{
        download_and_insert_dual_index_sets, download_and_insert_single_index_sets,
    },
    validate::Validate,
};

mod app_admin;
mod index_sets;
mod institution;
mod multiplexing_tags;
mod tenx_assays;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InitialData {
    institution: InstitutionCreation,
    app_admin: PersonCreation,
    single_index_set_urls: Vec<Url>,
    dual_index_set_urls: Vec<Url>,
    tenx_assays: Vec<TenxAssayCreation>,
    multiplexing_tags: Vec<MultiplexingTagCreation>,
}

impl InitialData {
    pub fn institution(&self) -> &InstitutionCreation {
        &self.institution
    }

    pub fn app_admin(&self) -> &PersonCreation {
        &self.app_admin
    }

    pub fn single_index_set_urls(&self) -> &[Url] {
        &self.single_index_set_urls
    }

    pub fn dual_index_set_urls(&self) -> &[Url] {
        &self.dual_index_set_urls
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
        single_index_set_urls,
        dual_index_set_urls,
        tenx_assays,
        multiplexing_tags,
    } = initial_data;

    let simple_operations = |db_conn: &mut PgConnection| -> Result<(), anyhow::Error> {
        institution.upsert(db_conn)?;
        app_admin.upsert(db_conn)?;
        for assay in tenx_assays {
            assay.upsert(db_conn)?;
        }
        for tag in multiplexing_tags {
            tag.upsert(db_conn)?;
        }

        Ok(())
    };

    download_and_insert_single_index_sets(single_index_set_urls, http_client.clone(), &db_conn)
        .await?;
    download_and_insert_dual_index_sets(dual_index_set_urls, http_client, &db_conn).await?;

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
