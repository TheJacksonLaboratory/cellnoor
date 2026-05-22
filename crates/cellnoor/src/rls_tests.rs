mod project_tests {
    use cellnoor_types::project::{NewProject, ProjectDetailed, ProjectQuery};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::projects::{
            create::test::insert_test_project, index_detailed::select_projects_detailed,
            show::select_project_by_id,
        },
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_project(tx: &db::Transaction<'_>) -> (NewProject, ProjectDetailed) {
        insert_test_project(&tx, |_| ()).await.unwrap()
    }

    pub async fn insert_inaccessible_project(
        tx: &db::Transaction<'_>,
    ) -> (NewProject, ProjectDetailed) {
        insert_test_project(&tx, |p| p.people = vec![])
            .await
            .unwrap()
    }

    async fn test_user_can_only_see_accessible_project(
        tx: &db::Transaction<'_>,
        accessible_project: ProjectDetailed,
    ) {
        let projects = select_projects_detailed(&tx, &mut ProjectQuery::default())
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

        let person_id = accessible_project.record.people[0];

        // Log in as the new user
        let mut client = db_client_as_user(person_id).await;
        let tx = client.begin().await.unwrap();

        test_user_can_only_see_accessible_project(&tx, accessible_project).await;
        test_user_cannot_see_inaccessible_project(&tx, *inaccessible_project.record.project.id)
            .await;
    }
}

mod specimen_tests {
    use cellnoor_types::specimen::{SpecimenDetailed, SpecimenQuery, creation::NewSpecimen};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::show::select_project_by_id,
            specimens::{
                create::test::insert_test_specimen_and_project,
                index_detailed::select_specimens_detailed, show::select_specimen_by_id,
            },
        },
        rls_tests::project_tests::insert_inaccessible_project,
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_specimen(
        tx: &db::Transaction<'_>,
    ) -> (NewSpecimen, SpecimenDetailed) {
        // The underlying `insert_test_project` adds one person to the project, so we
        // don't need to do anything extra here
        insert_test_specimen_and_project(&tx, |_| ()).await.unwrap()
    }

    pub async fn insert_inaccessible_specimen(
        tx: &db::Transaction<'_>,
    ) -> (NewSpecimen, SpecimenDetailed) {
        let (_, project) = insert_inaccessible_project(tx).await;

        insert_test_specimen_and_project(&tx, |s| {
            s.common_mut().project_id = *project.record.project.id
        })
        .await
        .unwrap()
    }

    pub async fn get_user_id_from_specimen(
        tx: &db::Transaction<'_>,
        specimen: &SpecimenDetailed,
    ) -> Uuid {
        let project = select_project_by_id(&tx, specimen.record.project_id)
            .await
            .unwrap();

        project.record.people[0]
    }

    async fn test_user_can_only_see_accessible_specimen(
        tx: &db::Transaction<'_>,
        accessible_specimen: SpecimenDetailed,
    ) {
        let specimens = select_specimens_detailed(&tx, &mut SpecimenQuery::default())
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
        test_user_cannot_see_inaccessible_specimen(&tx, *inaccessible_specimen.record.id).await;
    }
}

mod suspension_tests {
    use cellnoor_types::suspension::{NewSuspension, SuspensionDetailed, SuspensionQuery};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            projects::show::select_project_by_id,
            suspensions::{
                create::test::insert_test_suspension_and_specimen,
                index_detailed::select_suspensions_detailed, show::select_suspension_by_id,
            },
        },
        rls_tests::specimen_tests::insert_inaccessible_specimen,
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    async fn insert_accessible_suspension(
        tx: &db::Transaction<'_>,
    ) -> (NewSuspension, SuspensionDetailed) {
        // The underlying `insert_test_project` adds one person to the project, so we
        // don't need to do anything extra here
        insert_test_suspension_and_specimen(&tx, |_| ())
            .await
            .unwrap()
    }

    async fn insert_inaccessible_suspension(
        tx: &db::Transaction<'_>,
    ) -> (NewSuspension, SuspensionDetailed) {
        let (_, specimen) = insert_inaccessible_specimen(tx).await;

        insert_test_suspension_and_specimen(&tx, |s| s.record.specimen_id = *specimen.record.id)
            .await
            .unwrap()
    }

    async fn get_user_id_from_suspension(
        tx: &db::Transaction<'_>,
        suspension: &SuspensionDetailed,
    ) -> Uuid {
        let project = select_project_by_id(&tx, suspension.specimen.record.project_id)
            .await
            .unwrap();

        project.record.people[0]
    }

    async fn test_user_can_only_see_accessible_suspension(
        tx: &db::Transaction<'_>,
        accessible_suspension: SuspensionDetailed,
    ) {
        // Note that by querying the detailed view, we are querying
        // `suspension_detailed`, and that allows us to test whether a view built on top
        // of a security_invoker = true view still adheres to RLS
        let suspensions = select_suspensions_detailed(&tx, &mut SuspensionQuery::default())
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
        test_user_cannot_see_inaccessible_suspension(&tx, *inaccessible_suspension.record.id).await;
    }
}

mod suspension_pool_tests {
    use cellnoor_types::suspension_pool::SuspensionPoolQuery;

    use crate::{
        handlers::suspension_pools::{
            create::test::insert_test_suspension_pool_and_suspensions,
            index_compact::select_suspension_pools_compact,
        },
        state::test_util::{db_client_as_admin, db_client_as_user},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn user_can_access_suspension_pools() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, pool) = insert_test_suspension_pool_and_suspensions(&tx, |_| ())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let mut client = db_client_as_user(pool.preparers[0]).await;
        let tx = client.begin().await.unwrap();

        // As long as we don't get a permission_denied error, we are good
        select_suspension_pools_compact(&tx, &mut SuspensionPoolQuery::default())
            .await
            .unwrap();
    }
}
