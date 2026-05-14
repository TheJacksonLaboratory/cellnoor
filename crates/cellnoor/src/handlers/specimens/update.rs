use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{
    Specimen,
    creation::{NewSpecimen, NewSpecimenRecord},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, ToUpdateClause},
    },
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        specimens::{
            measurements::create::insert_specimen_measurement, show::select_specimen_by_id,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(record): Json<NewSpecimen>,
) -> Result<Json<Specimen>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_specimen_by_id(&tx, id, record).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: NewSpecimen,
) -> Result<Specimen, ErrorInner> {
    let (record, measurements) = record.split_for_insertion();

    update_specimen_record(tx, id, &record).await?;

    futures::future::try_join_all(
        measurements
            .iter()
            .map(|m| insert_specimen_measurement(tx, id, m)),
    )
    .await?;

    select_specimen_by_id(tx, id).await
}

async fn update_specimen_record(
    tx: &db::Transaction<'_>,
    id: Uuid,
    record: &NewSpecimenRecord,
) -> Result<(), ErrorInner> {
    let fields = record.as_field_value_pairs();

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update specimen set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::convert::identity;

    use cellnoor_types::specimen::{
        Fixative, SavedSpecimenRecord, Species, SpecimenType, ThermalPreservationMethod,
        creation::{
            NewSpecimen, NewSpecimenCommonFields,
            block::{BlockEmbeddingMatrix, BlockFixative, NewBlock},
        },
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::specimens::{
            create::test::insert_test_specimen_and_project, update::update_specimen_by_id,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let original_specimen = insert_test_specimen_and_project(&tx, identity).await;
        let original_record = original_specimen.record();

        let new_readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        let new_name = "updated".to_nonempty_string();
        let new_received_at = Timestamp::now();
        let new_tissue = "updated tissue".to_nonempty_string();

        let new_data = NewSpecimen::Block(NewBlock::Paraffin {
            inner: NewSpecimenCommonFields {
                readable_id: new_readable_id.clone(),
                name: new_name.clone(),
                submitted_by: original_record.submitted_by,
                received_at: new_received_at,
                project_id: original_record.project_id,
                species: Species::HomoSapiens,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: new_tissue.clone(),
                additional_data: None,
                measurements: vec![],
            },
            fixative: BlockFixative::FormaldehydeDerivative,
        });

        let updated = update_specimen_by_id(&tx, *original_record.id, new_data)
            .await
            .unwrap();

        assert_eq!(
            updated.record(),
            &SavedSpecimenRecord {
                id: original_record.id,
                readable_id: new_readable_id,
                name: new_name,
                submitted_by: original_record.submitted_by,
                project_id: original_record.project_id,
                received_at: new_received_at,
                species: Species::HomoSapiens,
                host_species: None,
                returned_at: None,
                returned_by: None,
                type_: SpecimenType::Block,
                embedded_in: Some(BlockEmbeddingMatrix::Paraffin),
                fixative: Some(Fixative::FormaldehydeDerivative),
                thermal_preservation_method: None,
                tissue: new_tissue,
                additional_data: None,
            }
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_missing() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new_data = NewSpecimen::Block(NewBlock::CarboxymethylCellulose {
            inner: NewSpecimenCommonFields {
                readable_id: "missing".to_nonempty_string(),
                name: "missing".to_nonempty_string(),
                submitted_by: Uuid::new_v4(),
                received_at: Timestamp::now(),
                project_id: Uuid::new_v4(),
                species: Species::MusMusculus,
                host_species: None,
                returned_by: None,
                returned_at: None,
                tissue: "tissue".to_nonempty_string(),
                additional_data: None,
                measurements: vec![],
            },
            fixative: None,
        });

        let error = update_specimen_by_id(&tx, Uuid::new_v4(), new_data)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }
}
