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
        SavedSpecimenRecord, Species, Specimen,
        creation::{
            NewSpecimen, NewSpecimenCommonFields,
            block::{BlockFixative, NewBlock},
        },
    };
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        error::ErrorInner,
        handlers::{
            projects::show::select_project_by_id,
            specimens::{
                create::test::insert_test_specimen_and_project, update::update_specimen_by_id,
            },
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            mut pre_update,
            Specimen::Detailed {
                record: SavedSpecimenRecord { id, .. },
                ..
            },
        ) = insert_test_specimen_and_project(&tx, identity).await
        else {
            panic!("expected Specimen::Detailed");
        };

        // Switch the variant to Paraffin (different derived type_/embedded_in/etc.)
        // and rename it. The helper produces a Cmc variant; we destructure the
        // common fields and re-wrap.
        pre_update.inner_mut().readable_id = Uuid::new_v4().to_string().to_nonempty_string();
        let inner = pre_update.into_inner();
        let pre_update = NewSpecimen::Block(NewBlock::Paraffin {
            inner,
            fixative: BlockFixative::FormaldehydeDerivative,
        });

        let Specimen::Detailed {
            record: post_update_record,
            project: post_update_project,
            measurements: post_update_measurements,
            links: _,
        } = update_specimen_by_id(&tx, *id, pre_update.clone())
            .await
            .unwrap()
        else {
            panic!("expected Specimen::Detailed");
        };

        let (input_record, _) = pre_update.split_for_insertion();

        let expected_record = SavedSpecimenRecord {
            id,
            readable_id: input_record.readable_id,
            name: input_record.name,
            submitted_by: input_record.submitted_by,
            project_id: input_record.project_id,
            received_at: input_record.received_at,
            species: input_record.species,
            host_species: input_record.host_species,
            returned_at: input_record.returned_at,
            returned_by: input_record.returned_by,
            type_: input_record.type_,
            embedded_in: input_record.embedded_in,
            fixative: input_record.fixative,
            thermal_preservation_method: input_record.thermal_preservation_method,
            tissue: input_record.tissue,
            additional_data: input_record.additional_data,
        };

        assert_eq!(post_update_record, expected_record);
        assert_eq!(
            post_update_project,
            select_project_by_id(&tx, expected_record.project_id)
                .await
                .unwrap()
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
