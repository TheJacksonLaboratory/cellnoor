use std::collections::HashMap;

use anyhow::ensure;
use cellnoor_schema::dual_index_sets;
use diesel::{RunQueryDsl, prelude::*};

use crate::initial_data::{
    Upsert,
    index_sets::common::{DNA_REGEX, INDEX_SET_NAME_REGEX, IndexSetName, insert_kit_name},
};

#[derive(serde::Deserialize)]
pub(super) struct DualIndexSet {
    #[serde(alias = "index(i7)")]
    index_i7: String,
    #[serde(alias = "index2_workflow_a(i5)")]
    index2_workflow_a_i5: String,
    #[serde(alias = "index2_workflow_b(i5)")]
    index2_workflow_b_i5: String,
}

impl DualIndexSet {
    fn validate(&self) -> anyhow::Result<()> {
        let Self {
            index_i7,
            index2_workflow_a_i5,
            index2_workflow_b_i5,
        } = self;

        let sequences = [index_i7, index2_workflow_a_i5, index2_workflow_b_i5];

        ensure!(
            sequences.iter().all(|s| DNA_REGEX.is_match(s)),
            "invalid DNA sequences: {sequences:?}"
        );

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = dual_index_sets, check_for_backend(diesel::pg::Pg))]
struct DualIndexSetInsertion<'a> {
    name: &'a str,
    kit: &'a str,
    well: &'a str,
    index_i7: &'a str,
    index2_workflow_a_i5: &'a str,
    index2_workflow_b_i5: &'a str,
}

#[allow(clippy::implicit_hasher)]
impl Upsert for HashMap<String, DualIndexSet> {
    fn upsert(self, db_conn: &mut PgConnection) -> anyhow::Result<()> {
        self.values().try_for_each(DualIndexSet::validate)?;

        let Some(index_set_name) = self.keys().next().cloned() else {
            return Ok(());
        };

        ensure!(
            INDEX_SET_NAME_REGEX.is_match(&index_set_name),
            "index set name does not match required pattern"
        );

        let kit_name = index_set_name.kit_name()?;
        insert_kit_name(kit_name, db_conn)?;

        let mut insertables = Vec::with_capacity(self.len());
        for (
            index_set_name,
            DualIndexSet {
                index_i7,
                index2_workflow_a_i5,
                index2_workflow_b_i5,
            },
        ) in &self
        {
            let well_name = index_set_name.well_name()?;

            insertables.push(DualIndexSetInsertion {
                name: index_set_name,
                kit: kit_name,
                well: well_name,
                index_i7,
                index2_workflow_a_i5,
                index2_workflow_b_i5,
            });
        }

        diesel::insert_into(dual_index_sets::table)
            .values(insertables)
            .on_conflict_do_nothing()
            .execute(db_conn)?;

        Ok(())
    }
}
