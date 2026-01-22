use std::{fmt::Display, str::FromStr, sync::Arc};

use anyhow::{Context, bail, ensure};
use cellnoor_models::{
    institution::{Institution, InstitutionCreation},
    multiplexing_tag::MultiplexingTagCreation,
    person::PersonCreation,
    tenx_assay::TenxAssayCreation,
};
use diesel::PgConnection;
pub(crate) use index_sets::IndexSetName;
use url::Url;

use crate::{
    api::{AuthorizedRequest, Request, auth::AuthorizationData},
    db::DbConnection,
    initial_data::index_sets::{
        download_and_insert_dual_index_sets, download_and_insert_single_index_sets,
    },
    state::AppState,
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

    async fn validate(self, db_pool: deadpool_diesel::postgres::Pool) -> anyhow::Result<Self> {
        let Self {
            institution,
            app_admin,
            single_index_set_urls,
            dual_index_set_urls,
            tenx_assays,
            multiplexing_tags,
        } = self;

        let validation_data = institution
            .fetch_validation_data(db_pool.get().await?)
            .await?;
        institution
            .validate(validation_data)
            .context("failed to validate institution in initial data")?;

        // let validation_data = app_admin
        //     .fetch_validation_data(db_pool.get().await?)
        //     .await?;
        // app_admin
        //     .validate(validation_data)
        //     .context("failed to validate app admin in initial data")?;

        single_index_set_urls
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        dual_index_set_urls
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        // tenx_assays.iter().try_for_each(|a| a.validate(db_conn))?;

        Ok(Self {
            institution,
            app_admin,
            single_index_set_urls,
            dual_index_set_urls,
            tenx_assays,
            multiplexing_tags,
        })
    }
}

pub async fn insert_initial_data(
    initial_data: InitialData,
    http_client: reqwest::Client,
    db_pool: deadpool_diesel::postgres::Pool,
) -> anyhow::Result<()> {
    let initial_data = initial_data.validate(db_pool.clone()).await?;

    let InitialData {
        institution,
        app_admin,
        single_index_set_urls,
        dual_index_set_urls,
        tenx_assays,
        multiplexing_tags,
    } = initial_data;

    let db_conn = db_pool.get().await?;

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

pub(super) fn validate_10x_genomics_url<S: AsRef<str> + Display>(url: &S) -> anyhow::Result<()> {
    let url = Url::from_str(url.as_ref())?;

    let Some(domain) = url.domain() else {
        bail!("URL must have domain");
    };

    ensure!(
        domain == "www.10xgenomics.com" || domain == "cdn.10xgenomics.com",
        "index sets must be downloaded from 10X Genomics URL"
    );

    Ok(())
}
