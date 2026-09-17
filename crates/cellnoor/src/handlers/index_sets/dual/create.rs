use std::collections::HashMap;

use axum::{Json, extract::State};
use cellnoor_types::{Relation, index_set::NewDualIndexSet};

use crate::{
    auth::AuthUser,
    db::{self, FieldValues, Insert},
    error::{Error, ErrorInner},
    handlers::index_sets::{
        NewIndexKit,
        index_set_name::{IndexKitName, IndexSetName, IndexSetWellName},
        insert_index_kit,
        sequence::DnaSequence,
    },
    state::AppState,
};

pub async fn create_dual_index_sets(
    State(state): State<AppState>,
    user: AuthUser,
    Json(sets): Json<HashMap<String, NewDualIndexSet>>,
) -> Result<Json<()>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    insert_dual_index_sets(&tx, &sets).await?;

    tx.commit().await?;

    Ok(Json(()))
}

async fn insert_dual_index_sets(
    tx: &db::Transaction<'_>,
    sets: &HashMap<String, NewDualIndexSet>,
) -> Result<(), ErrorInner> {
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
            return Err(ErrorInner::DataConstraint {
                resource: Some("dual_index_set".to_owned()),
                field: Some("name".to_owned()),
                message: "all index sets must share the same kit name".to_owned(),
                detail: None,
            });
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
) -> Result<(), ErrorInner> {
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

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use cellnoor_types::index_set::NewDualIndexSet;

    use crate::{
        db::{self, Sql},
        error::ErrorInner,
        handlers::index_sets::dual::create::insert_dual_index_sets,
        state::test_util::db_client_as_admin,
    };

    pub const DUAL_INDEX_SET_NAME: &str = "SI-TT-A1";

    pub async fn insert_test_dual_index_set(
        tx: &db::Transaction<'_>,
    ) -> Result<String, ErrorInner> {
        let name = DUAL_INDEX_SET_NAME.to_owned();

        // Acquire a db lock to prevent a concurrency bug during testing
        static LOCK_TABLE: &str = "select pg_advisory_xact_lock(hashtext($1))";

        let table = "dual_index_set";
        tx.execute(&Sql::new(LOCK_TABLE, vec![&table])).await?;

        let sql = Sql::new("select count(*) from dual_index_set", Vec::new());

        let n: i64 = tx.query_one_into(&sql).await.unwrap();

        if n > 0 {
            return Ok(name);
        }

        insert_dual_index_sets(
            tx,
            &HashMap::from_iter([(
                name.clone(),
                NewDualIndexSet {
                    index_i7: "GTAACATGCG".to_owned(),
                    index2_workflow_a_i5: "AGTGTTACCT".to_owned(),
                    index2_workflow_b_i5: "AGGTAACACT".to_owned(),
                },
            )]),
        )
        .await?;

        Ok(name)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_dual_index_set(&tx).await.unwrap();
    }
}
