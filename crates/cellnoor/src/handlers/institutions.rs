use cellnoor_types::{
    SimpleLinks,
    institution::{Institution, InstitutionRecord},
};

use crate::db::util::FromRecord;

pub mod create;
pub mod index;
pub mod show;

impl FromRecord<InstitutionRecord> for Institution {
    fn from_record(record: InstitutionRecord) -> Self {
        let id = record.id;
        Self {
            record,
            links: SimpleLinks {
                self_: format!("/institutions/{id}"),
            },
        }
    }
}
