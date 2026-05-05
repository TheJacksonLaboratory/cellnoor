use cellnoor_types::{
    SimpleLinks,
    person::{Person, PersonLinks, PersonRecord},
};

use crate::db::util::FromRecord;

pub mod create;
pub mod index;
pub mod show;

impl FromRecord<PersonRecord> for Person {
    fn from_record(record: PersonRecord) -> Self {
        let id = record.id;
        Self {
            record,
            links: PersonLinks {
                simple: SimpleLinks {
                    self_: format!("/people/{id}"),
                },
                projects: format!("/people/{id}/projects"),
            },
        }
    }
}
