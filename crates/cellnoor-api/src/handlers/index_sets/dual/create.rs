use std::collections::HashMap;

use axum::{Json, extract::State};
use cellnoor_types::{Relation, index_set::NewDualIndexSet};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::index_sets::{
        IndexSetError, NewIndexKit,
        index_set_name::{IndexKitName, IndexSetName, IndexSetWellName},
        insert_index_kit,
        sequence::DnaSequence,
    },
    state::AppState,
};

pub async fn create_dual_index_sets(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(sets): crate::extract::JsonExtractor<
        HashMap<String, NewDualIndexSet>,
    >,
) -> Result<Json<()>, IndexSetError> {
    state
        .in_transaction(user, async |tx| insert_dual_index_sets(tx, &sets).await)
        .await
}

async fn insert_dual_index_sets(
    tx: &db::Transaction<'_>,
    sets: &HashMap<String, NewDualIndexSet>,
) -> Result<(), IndexSetError> {
    let Some(first_index_set_name) = sets.keys().map(|name| IndexSetName::new(name)).next() else {
        return Ok(());
    };

    let first_kit_name = first_index_set_name?.kit_name();

    insert_index_kit(
        tx,
        &NewIndexKit {
            name: first_kit_name,
        },
    )
    .await?;

    let mut index_set_insertions = Vec::with_capacity(sets.len());
    for (
        index_set_name,
        NewDualIndexSet {
            index_i7,
            index2_workflow_a_i5,
            index2_workflow_b_i5,
        },
    ) in sets
    {
        let index_set_name = IndexSetName::new(index_set_name)?;
        let kit_name = index_set_name.kit_name();

        if kit_name != first_kit_name {
            return Err(IndexSetError::MixedKitNames);
        }

        let record = NewDualIndexSetRecord {
            name: index_set_name,
            kit: kit_name,
            well: index_set_name.well_name(),
            index_i7: DnaSequence::new(index_i7)?,
            index2_workflow_a_i5: DnaSequence::new(index2_workflow_a_i5)?,
            index2_workflow_b_i5: DnaSequence::new(index2_workflow_b_i5)?,
        };

        index_set_insertions.push(insert_dual_index_set(tx, record));
    }

    futures::future::try_join_all(index_set_insertions).await?;

    Ok(())
}

async fn insert_dual_index_set(
    tx: &db::Transaction<'_>,
    record: NewDualIndexSetRecord<'_>,
) -> Result<(), DbError> {
    tx.insert(&record).await?;

    Ok(())
}

struct NewDualIndexSetRecord<'a> {
    name: IndexSetName<'a>,
    kit: IndexKitName<'a>,
    well: IndexSetWellName<'a>,
    index_i7: DnaSequence<'a>,
    index2_workflow_a_i5: DnaSequence<'a>,
    index2_workflow_b_i5: DnaSequence<'a>,
}

impl<'a> Relation for NewDualIndexSetRecord<'a> {
    const NAME: &'static str = "dual_index_set";
}

impl<'a> Insert for NewDualIndexSetRecord<'a> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            name,
            kit,
            well,
            index_i7,
            index2_workflow_a_i5,
            index2_workflow_b_i5,
        } = self;

        vec![
            ("name", name),
            ("kit", kit),
            ("well", well),
            ("index_i7", index_i7),
            ("index2_workflow_a_i5", index2_workflow_a_i5),
            ("index2_workflow_b_i5", index2_workflow_b_i5),
        ]
    }
}

#[cfg(any(test, feature = "dev"))]
pub const GENE_EXPRESSION_DUAL_INDEX_SET_NAME: &str = "SI-TT-A1";

#[cfg(any(test, feature = "dev"))]
pub const FLEX_DUAL_INDEX_SET_NAME: &str = "SI-TS-A1";

#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_dual_index_sets(tx: &db::Transaction<'_>) -> Result<(), IndexSetError> {
    use crate::db::Sql;

    // Acquire a db lock to prevent a concurrency bug during testing
    tx.lock_table("dual_index_set").await?;

    let sql = Sql::new("select count(*) from dual_index_set", Vec::new());

    let n: i64 = tx.query_one_into(&sql).await.unwrap();

    if n > 0 {
        return Ok(());
    }

    insert_dual_index_sets(
        tx,
        &[(
            GENE_EXPRESSION_DUAL_INDEX_SET_NAME.to_owned(),
            NewDualIndexSet {
                index_i7: "GTAACATGCG".to_owned(),
                index2_workflow_a_i5: "AGTGTTACCT".to_owned(),
                index2_workflow_b_i5: "AGGTAACACT".to_owned(),
            },
        )]
        .into_iter()
        .collect(),
    )
    .await?;

    insert_dual_index_sets(
        tx,
        &[(
            FLEX_DUAL_INDEX_SET_NAME.to_owned(),
            NewDualIndexSet {
                index_i7: "AATTTCGGGT".to_owned(),
                index2_workflow_a_i5: "CTCTCTCCTC".to_owned(),
                index2_workflow_b_i5: "GAGGAGAGAG".to_owned(),
            },
        )]
        .into_iter()
        .collect(),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {

    use super::insert_test_dual_index_sets;
    use crate::state::dev_util::db_client_as_admin;

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_dual_index_sets(&tx).await.unwrap();
    }
}
