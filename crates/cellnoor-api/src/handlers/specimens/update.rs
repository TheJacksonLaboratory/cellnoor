use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::specimen::{SpecimenDetailed, creation::SpecimenUpdate};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        specimens::{
            convert_specimen_fields_to_record::convert_specimen_fields_to_record,
            show::select_specimen_by_id,
        },
    },
    state::AppState,
};

pub async fn update_specimen(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(update): crate::extract::JsonExtractor<SpecimenUpdate>,
) -> Result<Json<SpecimenDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| update_specimen_by_id(tx, id, update).await)
        .await
}

pub(in super::super) async fn update_specimen_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    update: SpecimenUpdate,
) -> Result<SpecimenDetailed, DbError> {
    tx.update(id, &convert_specimen_fields_to_record(update))
        .await?;

    select_specimen_by_id(tx, id).await
}

#[cfg(test)]
mod test {
    use cellnoor_types::{
        project::{ProjectActivePeriod, ProjectUpdate},
        specimen::{Species, creation::SpecimenOrigin},
    };
    use jiff::Timestamp;

    use crate::{
        db::DbError,
        handlers::{
            projects::update::update_project_by_id,
            specimens::{
                create::insert_test_specimen_and_project, show::select_specimen_by_id,
                update::update_specimen_by_id,
            },
        },
        state::dev_util::db_client_as_admin,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (insert_input, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let id = *inserted.record.id;

        let mut update = insert_input.simple;
        update.readable_id = crate::db::dummy_data::random_name_for("specimen");

        update_specimen_by_id(&tx, id, update).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ending_project_updates_specimen() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let project = inserted.project.record;

        let update = ProjectUpdate {
            name: project.name,
            active_period: ProjectActivePeriod {
                ended_at: Some(Timestamp::now()),
                ..project.active_period
            },
        };
        let updated_project = update_project_by_id(&tx, project.id, &update)
            .await
            .unwrap();

        let specimen = select_specimen_by_id(&tx, *inserted.record.id)
            .await
            .unwrap();

        assert_eq!(
            specimen.record.project_active_period,
            updated_project.record().active_period
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ending_project_before_specimen_received_fails() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, inserted) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let project = inserted.project.record;

        let update = ProjectUpdate {
            name: project.name,
            active_period: ProjectActivePeriod {
                ended_at: Some(project.active_period.started_at),
                ..project.active_period
            },
        };
        let error = update_project_by_id(&tx, project.id, &update)
            .await
            .unwrap_err();

        std::assert_matches!(
            error,
            DbError::DataConstraint { resource: Some(resource), .. } if resource == "specimen"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn host_species_update_reaches_child() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (parent_input, parent) = insert_test_specimen_and_project(&tx, |_| ()).await.unwrap();
        let (_, child) = insert_test_specimen_and_project(&tx, |sp| {
            sp.simple.origin = SpecimenOrigin::Derived {
                parent_specimen_id: *parent.record.id,
            };
        })
        .await
        .unwrap();

        let mut update = parent_input.simple;
        if let SpecimenOrigin::Primary { host_species, .. } = &mut update.origin {
            *host_species = Some(Species::HomoSapiens);
        }
        update_specimen_by_id(&tx, *parent.record.id, update)
            .await
            .unwrap();

        let child = select_specimen_by_id(&tx, *child.record.id).await.unwrap();

        assert_eq!(
            child.record.species.host_species,
            Some(Species::HomoSapiens)
        );
    }
}
