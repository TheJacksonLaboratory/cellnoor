use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, NewInstitution};

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{AsFieldValuePairs, FieldValuePairs, ToFieldListPlaceholdersParams},
    },
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_institution(&tx, &institution).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

impl AsFieldValuePairs<2> for NewInstitution {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, 2> {
        let Self {
            id: _,
            name,
            microsoft_entra_tenant_id,
        } = self;

        [
            ("name", name),
            ("microsoft_entra_tenant_id", microsoft_entra_tenant_id),
        ]
    }
}

pub async fn insert_institution(
    tx: &db::Transaction<'_>,
    new_record: &NewInstitution,
) -> Result<Institution, ErrorInner> {
    let fields = new_record.as_field_value_pairs();
    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    // Simple queries can be written inline
    let institution = tx
        .query_one_into(
            &format!(
                "insert into institution {field_list} values {placeholders} returning institution"
            ),
            &params,
        )
        .await
        .map(Institution::from_record)?;

    Ok(institution)
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        institution::{Institution, NewInstitution, SavedInstitutionRecord},
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        handlers::institutions::create::insert_institution,
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub async fn insert_test_institution<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewInstitution, Institution)
    where
        F: Fn(NewInstitution) -> NewInstitution,
    {
        let mut new = NewInstitution {
            id: NoId {},
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            microsoft_entra_tenant_id: Uuid::new_v4(),
        };

        new = modify(new);

        let inserted = insert_institution(tx, &new).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (
            input_record,
            Institution {
                record: output_record,
                links: _,
            },
        ) = insert_test_institution(&tx, identity).await;

        let expected = SavedInstitutionRecord {
            id: output_record.id,
            name: input_record.name,
            microsoft_entra_tenant_id: input_record.microsoft_entra_tenant_id,
        };

        assert_eq!(output_record, expected);
    }
}
