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
    operator::{StringOperator, TimestampOperator, UuidOperator},
    order_by::{OrderBy, OrderBySet},
    person::PersonQuery,
    project::{ProjectDetailed, ProjectField, ProjectPredicate, ProjectQuery},
};
use strum::VariantArray;

use crate::{
    extract::JsonExtractor,
    handlers::{
        IdParam,
        people::index_people,
        projects::{add_people_to_project, index_projects_detailed, remove_people_from_project},
    },
    state::AppState,
    ui::{
        BASE, Layout, MembersForm, UiError, User,
        format::person_options,
        params::{
            Labels, ListState, Opt, Paging, Param, Picker, end_of_day, parse_all, start_of_day,
            strings,
        },
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
pub(super) enum ProjectFilter {
    Name,
    CreatedBy,
    ActiveFrom,
    ActiveTo,
}

const SORT_FIELDS: &[(ProjectField, &str)] = &[
    (ProjectField::StartedAt, "Date started"),
    (ProjectField::EndedAt, "Date ended"),
    (ProjectField::Name, "Name"),
];

fn to_filter(list: &ListState<ProjectFilter>) -> Result<Filter<ProjectPredicate>, UiError> {
    let mut filters = Vec::new();

    for &field in ProjectFilter::VARIANTS {
        let values = list.selected(field);
        let date = list.last(Param::Value(field));

        let predicate = match field {
            ProjectFilter::Name => Some(ProjectPredicate::Name(
                StringOperator::TrgmAnyUnlessEmpty(strings(&values)),
            )),
            ProjectFilter::CreatedBy => Some(ProjectPredicate::CreatedBy(
                UuidOperator::InUnlessEmpty(parse_all(&values)?),
            )),
            ProjectFilter::ActiveFrom => date
                .map(start_of_day)
                .transpose()?
                .map(|from| ProjectPredicate::StartedAt(TimestampOperator::Gte(from))),
            ProjectFilter::ActiveTo => date
                .map(end_of_day)
                .transpose()?
                .map(|to| ProjectPredicate::EndedAt(TimestampOperator::Lte(to))),
        };

        filters.extend(predicate.map(Filter::from));
    }

    Ok(Filter::AllOf(filters))
}

#[derive(Template, WebTemplate)]
#[template(path = "projects.html")]
pub(super) struct ProjectsPage {
    layout: Layout,
    list: ListState<ProjectFilter>,
    order_by: OrderBy<ProjectField>,
    paging: Paging,
    projects: Vec<ProjectDetailed>,
    people: Vec<Opt>,
}

impl ProjectsPage {
    fn members(&self, project: &ProjectDetailed) -> Picker<&'static str> {
        Picker::members(&self.people, &project.record.members)
    }

    fn members_action(&self, project: &ProjectDetailed) -> String {
        format!("{BASE}/projects/{}/members", project.record.project.id)
    }
}

pub(super) async fn index(
    State(state): State<AppState>,
    User(user): User,
    Query(params): Query<HashMap<Param<ProjectFilter>, Vec<String>>>,
) -> Result<ProjectsPage, UiError> {
    let list = ListState::new(params);
    let order_by = list.order_by(SORT_FIELDS)?;
    let limit = list.limit()?;

    let query = ProjectQuery {
        filter: Some(to_filter(&list)?),
        limit,
        offset: 0,
        order_by: OrderBySet::One(order_by),
    };

    let (Json(projects), Json(people), layout) = tokio::try_join!(
        index_projects_detailed(State(state.clone()), user, JsonExtractor(query)),
        index_people(
            State(state.clone()),
            user,
            JsonExtractor(PersonQuery::default())
        ),
        Layout::load(&state, user),
    )?;

    Ok(ProjectsPage {
        layout,
        list,
        order_by,
        paging: Paging::new(limit, projects.len()),
        projects,
        people: person_options(&people),
    })
}

pub(super) async fn update_members(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(MembersForm { add, remove }): Form<MembersForm>,
) -> Result<Redirect, UiError> {
    if let Some(person) = add {
        let Json(_) = add_people_to_project(
            State(state.clone()),
            user,
            Path(IdParam { id }),
            JsonExtractor(vec![person]),
        )
        .await?;
    }

    if let Some(person) = remove {
        let Json(_) = remove_people_from_project(
            State(state),
            user,
            Path(IdParam { id }),
            JsonExtractor(vec![person]),
        )
        .await?;
    }

    Ok(redirect_back(&headers))
}
