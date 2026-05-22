mod project_tests {
    use cellnoor_types::project::{NewProject, Project, ProjectQuery};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::projects::{
            create::test::insert_test_project, index::select_projects, show::select_project_by_id,
        },
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_project(tx: &db::Transaction<'_>) -> (NewProject, Project) {
        insert_test_project(&tx, |_| ()).await.unwrap()
    }

    pub async fn insert_inaccessible_project(tx: &db::Transaction<'_>) -> (NewProject, Project) {
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
            projects::{create::test::insert_test_project, show::select_project_by_id},
            specimens::{
                create::test::insert_test_specimen_and_project, index::select_specimens,
                show::select_specimen_by_id,
            },
        },
        rls_tests::project_tests::insert_inaccessible_project,
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, Specimen) {
        // The underlying `insert_test_project` adds one person to the project, so we
        // don't need to do anything extra here
        insert_test_specimen_and_project(&tx, |_| ()).await.unwrap()
    }

    pub async fn insert_inaccessible_specimen(tx: &db::Transaction<'_>) -> (NewSpecimen, Specimen) {
        let (_, project) = insert_inaccessible_project(tx).await;

        insert_test_specimen_and_project(&tx, |s| s.common_mut().project_id = *project.record().id)
            .await
            .unwrap()
    }

    pub async fn get_user_id_from_specimen(tx: &db::Transaction<'_>, specimen: &Specimen) -> Uuid {
        let Project::Detailed { record, .. } =
            select_project_by_id(&tx, specimen.record().project_id)
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

mod suspension_tests {
    use cellnoor_types::{
        project::Project,
        specimen::{Specimen, SpecimenQuery, creation::NewSpecimen},
        suspension::{NewSuspension, Suspension, SuspensionQuery},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::{create::test::insert_test_project, show::select_project_by_id},
            specimens::{
                create::test::insert_test_specimen_and_project, index::select_specimens,
                show::select_specimen_by_id,
            },
            suspensions::{
                create::test::insert_test_suspension_and_specimen, index::select_suspensions,
                show::select_suspension_by_id,
            },
        },
        rls_tests::specimen_tests::insert_inaccessible_specimen,
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_suspension(tx: &db::Transaction<'_>) -> (NewSuspension, Suspension) {
        // The underlying `insert_test_project` adds one person to the project, so we
        // don't need to do anything extra here
        insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap()
    }

    async fn insert_inaccessible_suspension(
        tx: &db::Transaction<'_>,
    ) -> (NewSuspension, Suspension) {
        let (_, specimen) = insert_inaccessible_specimen(tx).await;

        insert_test_suspension_and_specimen(&tx, |s| s.record.specimen_id = *specimen.record().id)
            .await
            .unwrap()
    }

    async fn get_user_id_from_suspension(
        tx: &db::Transaction<'_>,
        suspension: &Suspension,
    ) -> Uuid {
        let Suspension::Detailed { specimen, .. } = suspension else {
            unreachable!("insert_specimen returns Specimen::Detailed")
        };

        let Project::Detailed { record, .. } =
            select_project_by_id(&tx, *&specimen.record().project_id)
                .await
                .unwrap()
        else {
            unreachable!()
        };

        record.people[0]
    }

    async fn test_user_can_only_see_accessible_suspension(
        tx: &db::Transaction<'_>,
        accessible_suspension: Suspension,
    ) {
        let suspensions = select_suspensions(
            &tx,
            &mut SuspensionQuery {
                detailed: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(suspensions, vec![accessible_suspension]);
    }

    async fn test_user_cannot_see_inaccessible_suspension(
        tx: &db::Transaction<'_>,
        inaccessible_suspension_id: Uuid,
    ) {
        // Check that the inaccessible project causes a `ResourceNotFound`
        let error = select_suspension_by_id(&tx, inaccessible_suspension_id)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn row_level_security_for_suspensions() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Insert a specimen the user can access and one the user cannot
        let ((_, accessible_suspension), (_, inaccessible_suspension)) = tokio::join!(
            insert_accessible_suspension(&tx),
            insert_inaccessible_suspension(&tx)
        );
        let user_id = get_user_id_from_suspension(&tx, &accessible_suspension).await;

        // Commit this transaction so the change persists for the next part of the test
        tx.commit().await.unwrap();

        // Log in as the new user
        let mut client = db_client_as_user(user_id).await;
        let tx = client.begin().await.unwrap();

        test_user_can_only_see_accessible_suspension(&tx, accessible_suspension).await;
        test_user_cannot_see_inaccessible_suspension(&tx, *inaccessible_suspension.record().id)
            .await;
    }
}

mod suspension_pool_tests {
    use cellnoor_types::{
        project::Project,
        specimen::{Specimen, SpecimenQuery, creation::NewSpecimen},
        suspension::{NewSuspension, Suspension, SuspensionQuery},
        suspension_pool::{SuspensionPool, SuspensionPoolQuery},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::{create::test::insert_test_project, show::select_project_by_id},
            specimens::{
                create::test::insert_test_specimen_and_project, index::select_specimens,
                show::select_specimen_by_id,
            },
            suspension_pools::{
                create::test::insert_test_suspension_pool_and_suspensions,
                index::select_suspension_pools,
            },
            suspensions::{
                create::test::insert_test_suspension_and_specimen, index::select_suspensions,
                show::select_suspension_by_id,
            },
        },
        rls_tests::specimen_tests::insert_inaccessible_specimen,
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn user_can_access_suspension_pools() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, SuspensionPool::Detailed { preparers, .. }) =
            insert_test_suspension_pool_and_suspensions(&tx, |_| ())
                .await
                .unwrap()
        else {
            unreachable!()
        };
        tx.commit().await.unwrap();

        let mut client = db_client_as_user(preparers[0]).await;
        let tx = client.begin().await.unwrap();

        // As long as we don't get a permission_denied error, we are good
        select_suspension_pools(&tx, &mut SuspensionPoolQuery::default())
            .await
            .unwrap();
    }
}
