mod project_tests {
    use cellnoor_types::project::{NewProject, Project, ProjectQuery};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::projects::{
                create::test::insert_test_project, index::select_projects,
                show::select_project_by_id,
            },
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_project(tx: &db::Transaction<'_>) -> (NewProject, Project) {
        insert_test_project(&tx, |_| ()).await.unwrap()
    }

    async fn insert_inaccessible_project(tx: &db::Transaction<'_>) -> (NewProject, Project) {
        insert_test_project(&tx, |p| p.people = vec![])
            .await
            .unwrap()
    }

    async fn test_user_can_only_see_accessible_project(
        tx: &db::Transaction<'_>,
        accessible_project: Project,
    ) {
        let projects = select_projects(
            &tx,
            &mut ProjectQuery {
                detailed: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(projects, vec![accessible_project]);
    }

    async fn test_user_cannot_see_inaccessible_project(
        tx: &db::Transaction<'_>,
        inaccessible_project_id: Uuid,
    ) {
        // Check that the inaccessible project causes a `ResourceNotFound`
        let error = select_project_by_id(&tx, inaccessible_project_id)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn row_level_security_for_projects() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert a project the user can access and one the user cannot
        let ((_, accessible_project), (_, inaccessible_project)) = tokio::join!(
            insert_accessible_project(&tx),
            insert_inaccessible_project(&tx)
        );

        // We have to commit this transaction so the change persists for the next part
        // of the test
        tx.commit().await.unwrap();

        let Project::Detailed { record, .. } = &accessible_project else {
            unreachable!("insert_project should return Project::Detailed");
        };

        // Log in as the new user
        let mut client = db_client_as_user(record.people[0]).await;
        let tx = client.begin().await.unwrap();

        test_user_can_only_see_accessible_project(&tx, accessible_project).await;
        test_user_cannot_see_inaccessible_project(&tx, *inaccessible_project.record().id).await;
    }
}

mod specimen_tests {
    use cellnoor_types::{
        project::Project,
        specimen::{Specimen, SpecimenQuery, creation::NewSpecimen},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::{
                create::test::insert_test_project,
                show::select_project_by_id,
            },
            specimens::{
                create::test::insert_test_specimen_and_project, index::select_specimens,
                show::select_specimen_by_id,
            },
        },
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, Specimen) {
        // The underlying `insert_test_project` adds one person to the project, so we
        // don't need to do anything extra here
        insert_test_specimen_and_project(&tx, |_| ()).await.unwrap()
    }

    async fn insert_inaccessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, Specimen) {
        let (_, project) = insert_test_project(tx, |p| p.people = vec![])
            .await
            .unwrap();
        let project_id = *project.record().id;

        insert_test_specimen_and_project(&tx, |s| s.common_mut().project_id = project_id)
            .await
            .unwrap()
    }

    async fn get_user_id_from_specimen(tx: &db::Transaction<'_>, specimen: &Specimen) -> Uuid {
        let Specimen::Detailed { project, .. } = specimen else {
            unreachable!("insert_specimen returns Specimen::Detailed")
        };
        let Project::Detailed { record, .. } = select_project_by_id(&tx, *project.record().id)
            .await
            .unwrap()
        else {
            unreachable!()
        };

        record.people[0]
    }

    async fn test_user_can_only_see_accessible_specimen(
        tx: &db::Transaction<'_>,
        accessible_specimen: Specimen,
    ) {
        let specimens = select_specimens(
            &tx,
            &mut SpecimenQuery {
                detailed: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(specimens, vec![accessible_specimen]);
    }

    async fn test_user_cannot_see_inaccessible_specimen(
        tx: &db::Transaction<'_>,
        inaccessible_specimen_id: Uuid,
    ) {
        // Check that the inaccessible project causes a `ResourceNotFound`
        let error = select_specimen_by_id(&tx, inaccessible_specimen_id)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn row_level_security_for_specimens() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert a specimen the user can access and one the user cannot
        let ((_, accessible_specimen), (_, inaccessible_specimen)) = tokio::join!(
            insert_accessible_specimen(&tx),
            insert_inaccessible_specimen(&tx)
        );
        let user_id = get_user_id_from_specimen(&tx, &accessible_specimen).await;

        // Commit this transaction so the change persists for the next part of the test
        tx.commit().await.unwrap();

        // Log in as the new user
        let mut client = db_client_as_user(user_id).await;
        let tx = client.begin().await.unwrap();

        test_user_can_only_see_accessible_specimen(&tx, accessible_specimen).await;
        test_user_cannot_see_inaccessible_specimen(&tx, *inaccessible_specimen.record().id).await;
    }
}
