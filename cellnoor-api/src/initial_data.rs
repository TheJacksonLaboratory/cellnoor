use std::{fmt::Display, str::FromStr};

use anyhow::{Context, bail, ensure};
use cellnoor_models::{
    institution::NewInstitution, multiplexing_tag::NewMultiplexingTag, person::NewPerson,
    tenx_assay::NewTenxAssay,
};
use diesel_async::AsyncPgConnection;
use url::Url;

use crate::{
    api::util::validate_email,
    initial_data::index_sets::{
        download_and_insert_dual_index_sets, download_and_insert_single_index_sets,
    },
};

mod app_admin;
pub(crate) mod index_sets;
mod institution;
mod multiplexing_tags;
mod tenx_assays;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InitialData {
    institution: NewInstitution,
    app_admin: NewPerson,
    single_index_set_urls: Vec<Url>,
    dual_index_set_urls: Vec<Url>,
    tenx_assays: Vec<NewTenxAssay>,
    multiplexing_tags: Vec<NewMultiplexingTag>,
}

impl InitialData {
    pub fn institution(&self) -> &NewInstitution {
        &self.institution
    }

    pub fn app_admin(&self) -> &NewPerson {
        &self.app_admin
    }

    pub fn single_index_set_urls(&self) -> &[Url] {
        &self.single_index_set_urls
    }

    pub fn dual_index_set_urls(&self) -> &[Url] {
        &self.dual_index_set_urls
    }

    pub fn tenx_assays(&self) -> &[NewTenxAssay] {
        &self.tenx_assays
    }

    fn validate(self) -> anyhow::Result<Self> {
        let Self {
            institution,
            app_admin,
            single_index_set_urls,
            dual_index_set_urls,
            tenx_assays,
            multiplexing_tags,
        } = self;

        validate_email(app_admin.email())
            .context("failed to validate app admin in initial data")?;

        ensure!(
            app_admin.microsoft_entra_oid().is_some(),
            "app admin must have Microsoft Entra OID"
        );

        single_index_set_urls
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        dual_index_set_urls
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        tenx_assays
            .iter()
            .try_for_each(|a| validate_10x_genomics_url(&a.protocol_url()))?;

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
    db_conn: &AsyncPgConnection,
) -> anyhow::Result<()> {
    let initial_data = initial_data.validate()?;

    let InitialData {
        institution,
        app_admin,
        single_index_set_urls,
        dual_index_set_urls,
        tenx_assays,
        multiplexing_tags,
    } = initial_data;

    let upsert_assays = tenx_assays.into_iter().map(|a| a.upsert(&db_conn));
    let upsert_multiplexing_tags = multiplexing_tags.into_iter().map(|t| t.upsert(&db_conn));

    download_and_insert_single_index_sets(single_index_set_urls, http_client.clone(), db_conn)
        .await?;
    download_and_insert_dual_index_sets(dual_index_set_urls, http_client, db_conn).await?;

    tokio::try_join!(
        institution.upsert(&db_conn),
        app_admin.upsert(&db_conn),
        futures::future::try_join_all(upsert_assays),
        futures::future::try_join_all(upsert_multiplexing_tags)
    )?;

    Ok(())
}

trait Upsert {
    async fn upsert(self, db_conn: &AsyncPgConnection) -> anyhow::Result<()>;
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
