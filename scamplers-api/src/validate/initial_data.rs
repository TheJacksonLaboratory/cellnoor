use crate::{initial_data::InitialData, validate::Validate};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("app_admin must have Microsoft Entra OID")]
    AppAdminMicrosoftEntraOidError,
}

impl Validate for InitialData {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        self.institution().validate(db_conn)?;
        self.app_admin().validate(db_conn)?;
        if self.app_admin().microsoft_entra_oid().is_none() {
            return Err(Error::AppAdminMicrosoftEntraOidError)?;
        }
        // self.index_set_urls.validate(db_conn)?;

        Ok(())
    }
}
