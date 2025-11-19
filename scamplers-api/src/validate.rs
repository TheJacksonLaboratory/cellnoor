use diesel::PgConnection;

mod initial_data;
mod institution;
mod lab;
mod person;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "cause")]
#[error(transparent)]
pub enum Error {
    InsertInitialData(#[from] initial_data::Error),
    CreatePerson(#[from] person::Error),
    CreateLab(#[from] lab::Error),
}

pub trait Validate {
    fn validate(&self, _db_conn: &mut PgConnection) -> Result<(), Error> {
        Ok(())
    }
}
