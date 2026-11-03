use std::collections::HashMap;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Redirect,
};
use axum_extra::extract::{Form, Query};
use cellnoor_types::{
    filter::Filter,
    institution::InstitutionQuery,
    operator::{BoolOperator, StringOperator, UuidOperator},
    order_by::{OrderBy, OrderBySet},
    person::{
        Person, PersonField, PersonPredicate, PersonQuery, PersonSimpleFields, PersonUpdate,
        PersonWithPermissions,
    },
};
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    extract::JsonExtractor,
    handlers::{
        IdParam,
        institutions::index_institutions,
        people::{index_people_with_permissions, show_person, update_person},
    },
    state::AppState,
    ui::{
        BASE, Layout, PermissionGrid, UiError, User,
        format::staff_options,
        params::{Labels, ListState, Opt, Paging, Param, parse_all, strings},
        redirect_back,
    },
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub(super) enum PersonFilter {
    Name,
    Email,
    IsStaff,
    InstitutionId,
}

const SORT_FIELDS: &[(PersonField, &str)] =
    &[(PersonField::Name, "Name"), (PersonField::Email, "Email")];

fn to_filter(list: &ListState<PersonFilter>) -> Result<Filter<PersonPredicate>, UiError> {
    let mut filters = Vec::new();

    for &field in PersonFilter::VARIANTS {
        let values = list.selected(field);

        let predicate = match field {
            PersonFilter::Name => {
                PersonPredicate::Name(StringOperator::TrgmAnyUnlessEmpty(strings(&values)))
            }
            PersonFilter::Email => PersonPredicate::Email(StringOperator::LikeAnyUnlessEmpty(
                values.iter().map(|email| format!("%{email}%")).collect(),
            )),
            PersonFilter::IsStaff => {
                PersonPredicate::IsStaff(BoolOperator::InUnlessEmpty(parse_all(&values)?))
            }
            PersonFilter::InstitutionId => {
                PersonPredicate::InstitutionId(UuidOperator::InUnlessEmpty(parse_all(&values)?))
            }
        };

        filters.push(predicate.into());
    }

    Ok(Filter::AllOf(filters))
}

#[derive(Template, WebTemplate)]
#[template(path = "people.html")]
pub(super) struct PeoplePage {
    layout: Layout,
    list: ListState<PersonFilter>,
    order_by: OrderBy<PersonField>,
    paging: Paging,
    people: Vec<PersonWithPermissions>,
    institutions: Vec<Opt>,
    staff: Vec<Opt>,
}

impl PeoplePage {
    fn permissions(&self, person: &PersonWithPermissions) -> PermissionGrid {
        PermissionGrid::new(person.person.id, &person.permissions)
    }

    fn staff_action(&self, person: &PersonWithPermissions) -> String {
        format!("{BASE}/settings/people/{}/staff", person.person.id)
    }
}

pub(super) async fn index(
    State(state): State<AppState>,
    User(user): User,
    Query(params): Query<HashMap<Param<PersonFilter>, Vec<String>>>,
) -> Result<PeoplePage, UiError> {
    let list = ListState::new(params);
    let order_by = list.order_by(SORT_FIELDS)?;
    let limit = list.limit()?;

    let query = PersonQuery {
        filter: Some(to_filter(&list)?),
        limit,
        offset: 0,
        order_by: OrderBySet::One(order_by),
    };

    let (Json(people), Json(institutions), layout) = tokio::try_join!(
        index_people_with_permissions(State(state.clone()), user, JsonExtractor(query)),
        index_institutions(
            State(state.clone()),
            user,
            JsonExtractor(InstitutionQuery::default())
        ),
        Layout::load(&state, user),
    )?;

    Ok(PeoplePage {
        layout,
        list,
        order_by,
        paging: Paging::new(limit, people.len()),
        people,
        institutions: institutions
            .iter()
            .map(|institution| {
                Opt::new(
                    institution.record.id.to_string(),
                    institution.record.name.to_string(),
                )
            })
            .collect(),
        staff: staff_options(),
    })
}

#[derive(Deserialize)]
pub(super) struct StaffForm {
    is_staff: Option<String>,
}

pub(super) async fn set_staff(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(StaffForm { is_staff }): Form<StaffForm>,
) -> Result<Redirect, UiError> {
    let Json(Person { record, .. }) =
        show_person(State(state.clone()), user, Path(IdParam { id })).await?;

    let update = PersonUpdate {
        simple: PersonSimpleFields {
            name: record.name,
            institution_id: record.institution_id,
            is_staff: is_staff.is_some(),
            orcid: record.orcid,
        },
        email: record.email,
    };

    let Json(_) = update_person(State(state), user, Path(IdParam { id }), JsonExtractor(update)).await?;

    Ok(redirect_back(&headers))
}
