use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{
    NewSpecimen, Specimen, SpecimenCommonFields, SpecimenVariableFields,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToUpdateClause},
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
    let ((common_fields, measurements), variable_fields) = record.split_for_insertion();

    update_specimen_record(tx, id, &(common_fields, variable_fields)).await?;

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
    (
        SpecimenCommonFields {
            readable_id,
            name,
            submitted_by,
            received_at,
            project_id,
            species,
            host_species,
            returned_by,
            returned_at,
            tissue,
            additional_data,
            measurements: _,
        },
        SpecimenVariableFields {
            type_,
            embedded_in,
            fixative,
            thermal_preservation_method,
        },
    ): &(SpecimenCommonFields, SpecimenVariableFields),
) -> Result<(), ErrorInner> {
    let fields: FieldValuePairs<_> = [
        ("readable_id", readable_id),
        ("name", name),
        ("submitted_by", submitted_by),
        ("received_at", received_at),
        ("project_id", project_id),
        ("species", species),
        ("host_species", host_species),
        ("returned_by", returned_by),
        ("returned_at", returned_at),
        ("tissue", tissue),
        ("additional_data", additional_data),
        ("type", type_),
        ("embedded_in", embedded_in),
        ("fixative", fixative),
        ("thermal_preservation_method", thermal_preservation_method),
    ];

    let (update_clause, params) = fields.to_update_clause(&id);

    let n = tx
        .execute(&format!("update specimen set {update_clause}"), &params)
        .await?;

    if n == 0 {
        return Err(ErrorInner::ResourceNotFound.into());
    }

    Ok(())
}
