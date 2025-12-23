use std::{fmt::Display, str::FromStr};

use url::Url;

use crate::{initial_data::InitialData, validate::Validate};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "InitialDataValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("app_admin must have Microsoft Entra OID")]
    AppAdminWithoutMicrosoftEntraOid,
    #[error("URL '{0}' does not have domain '10xgenomics.com'")]
    Non10xGenomicsUrl(String),
}

impl Validate for InitialData {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        self.institution().validate(db_conn)?;
        self.app_admin().validate(db_conn)?;
        if self.app_admin().microsoft_entra_oid().is_none() {
            return Err(Error::AppAdminWithoutMicrosoftEntraOid)?;
        }
        self.single_index_set_urls()
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        self.dual_index_set_urls()
            .iter()
            .try_for_each(validate_10x_genomics_url)?;
        self.tenx_assays()
            .iter()
            .try_for_each(|a| a.validate(db_conn))?;

        Ok(())
    }
}

pub(super) fn validate_10x_genomics_url<S: AsRef<str> + Display>(
    url: &S,
) -> Result<(), super::Error> {
    let url = Url::from_str(url.as_ref())
        .map_err(|_| Error::Non10xGenomicsUrl(format!("failed to parse {url} as url")))?;

    let Some(domain) = url.domain() else {
        return Err(Error::Non10xGenomicsUrl(url.to_string()))?;
    };

    if !(domain == "www.10xgenomics.com" || domain == "cdn.10xgenomics.com") {
        Err(Error::Non10xGenomicsUrl(url.to_string()))?;
    }

    Ok(())
}
