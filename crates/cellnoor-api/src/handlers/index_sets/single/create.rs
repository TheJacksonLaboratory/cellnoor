use axum::{Json, extract::State};
use cellnoor_types::Relation;

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

pub async fn create_single_index_sets(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(sets): crate::extract::JsonExtractor<Vec<(String, [String; 4])>>,
) -> Result<Json<()>, IndexSetError> {
    state
        .in_transaction(user, async |tx| insert_single_index_sets(tx, &sets).await)
        .await
}

async fn insert_single_index_sets(
    tx: &db::Transaction<'_>,
    sets: &[(String, [String; 4])],
) -> Result<(), IndexSetError> {
    let Some(first_index_set_name) = sets.iter().map(|(name, _)| IndexSetName::new(name)).next()
    else {
        return Ok(());
    };

    let first_kit_name = first_index_set_name?.kit_name();
    let mut index_set_insertions = Vec::with_capacity(sets.len());

    for (index_set_name, sequences) in sets {
        let index_set_name = IndexSetName::new(index_set_name)?;
        let kit_name = index_set_name.kit_name();

        if kit_name != first_kit_name {
            return Err(IndexSetError::MixedKitNames);
        }

        let sequences: Vec<_> = sequences
            .iter()
            .map(|s| DnaSequence::new(s))
            .collect::<Result<Vec<_>, IndexSetError>>()?;

        let record = NewSingleIndexSetRecord {
            name: index_set_name,
            kit: kit_name,
            well: index_set_name.well_name(),
            sequences: *sequences.as_array().unwrap(),
        };

        index_set_insertions.push(insert_single_index_set(tx, record));
    }

    insert_index_kit(
        tx,
        &NewIndexKit {
            name: first_kit_name,
        },
    )
    .await?;

    futures::future::try_join_all(index_set_insertions).await?;

    Ok(())
}

async fn insert_single_index_set(
    tx: &db::Transaction<'_>,
    record: NewSingleIndexSetRecord<'_>,
) -> Result<(), DbError> {
    tx.insert(&record).await?;

    Ok(())
}

struct NewSingleIndexSetRecord<'a> {
    name: IndexSetName<'a>,
    kit: IndexKitName<'a>,
    well: IndexSetWellName<'a>,
    sequences: [DnaSequence<'a>; 4],
}

impl<'a> Relation for NewSingleIndexSetRecord<'a> {
    const NAME: &'static str = "single_index_set";
}

impl<'a> Insert for NewSingleIndexSetRecord<'a> {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        let Self {
            name,
            kit,
            well,
            sequences,
        } = self;

        vec![
            ("name", name),
            ("kit", kit),
            ("well", well),
            ("sequences", sequences),
        ]
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{
        db::{self, DbError},
        handlers::index_sets::{IndexSetError, single::create::insert_single_index_sets},
        state::dev_util::db_client_as_admin,
    };

    pub async fn insert_test_single_index_set(
        tx: &db::Transaction<'_>,
    ) -> Result<String, IndexSetError> {
        let name = "SI-GA-A1".to_owned();

        match insert_single_index_sets(
            tx,
            &[(
                name.clone(),
                ["GGTTTACT", "CTAAACGG", "TCGGCGTC", "AACCGTAA"].map(str::to_owned),
            )],
        )
        .await
        {
            Ok(_) | Err(IndexSetError::Db(DbError::DataConstraint { .. })) => Ok(name),
            Err(e) => Err(e),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_single_index_set(&tx).await.unwrap();
    }
}
