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
    bot_account::{
        BotAccount, BotAccountDetailed, BotAccountField, BotAccountPredicate, BotAccountQuery,
        BotAccountSimpleFields, BotAccountUpdate, NewBotAccount,
    },
    filter::Filter,
    nonempty::NonemptyString,
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    order_by::{OrderBy, OrderBySet},
    person::PersonQuery,
};
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    extract::JsonExtractor,
    handlers::{
        IdParam,
        bot_accounts::{
            add_people_to_bot_account, create_bot_account, index_bot_accounts_detailed,
            remove_people_from_bot_account, update_bot_account,
        },
        delete_resource,
        people::index_people,
    },
    state::AppState,
    ui::{
        BASE, Layout, MembersForm, PermissionGrid, UiError, User,
        format::{person_options, staff_options},
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
pub(super) enum BotAccountFilter {
    Description,
    IsStaff,
    OwnedBy,
    CreatedFrom,
    CreatedTo,
}

const SORT_FIELDS: &[(BotAccountField, &str)] = &[
    (BotAccountField::CreatedAt, "Date created"),
    (BotAccountField::Description, "Description"),
];

fn to_filter(list: &ListState<BotAccountFilter>) -> Result<Filter<BotAccountPredicate>, UiError> {
    let mut filters = Vec::new();

    for &field in BotAccountFilter::VARIANTS {
        let values = list.selected(field);
        let date = list.last(Param::Value(field));

        let predicate = match field {
            BotAccountFilter::Description => Some(BotAccountPredicate::Description(
                StringOperator::TrgmAnyUnlessEmpty(strings(&values)),
            )),
            BotAccountFilter::IsStaff => Some(BotAccountPredicate::IsStaff(
                BoolOperator::InUnlessEmpty(parse_all(&values)?),
            )),
            BotAccountFilter::OwnedBy => Some(BotAccountPredicate::OwnedBy(
                UuidOperator::InUnlessEmpty(parse_all(&values)?),
            )),
            BotAccountFilter::CreatedFrom => date
                .map(start_of_day)
                .transpose()?
                .map(|from| BotAccountPredicate::CreatedAt(TimestampOperator::Gte(from))),
            BotAccountFilter::CreatedTo => date
                .map(end_of_day)
                .transpose()?
                .map(|to| BotAccountPredicate::CreatedAt(TimestampOperator::Lte(to))),
        };

        filters.extend(predicate.map(Filter::from));
    }

    Ok(Filter::AllOf(filters))
}

#[derive(Template, WebTemplate)]
#[template(path = "bot_accounts.html")]
pub(super) struct BotAccountsPage {
    layout: Layout,
    list: ListState<BotAccountFilter>,
    order_by: OrderBy<BotAccountField>,
    paging: Paging,
    bot_accounts: Vec<BotAccountDetailed>,
    people: Vec<Opt>,
    staff: Vec<Opt>,
}

impl BotAccountsPage {
    fn title(&self, bot_account: &BotAccountDetailed) -> String {
        let BotAccount {
            id, description, ..
        } = &bot_account.bot_account;

        description
            .as_ref()
            .map_or_else(|| id.to_string(), ToString::to_string)
    }

    fn action(&self, bot_account: &BotAccountDetailed, action: &str) -> String {
        format!(
            "{BASE}/settings/bot-accounts/{}/{action}",
            bot_account.bot_account.id
        )
    }

    fn members(&self, bot_account: &BotAccountDetailed) -> Picker<&'static str> {
        Picker::members(&self.people, &bot_account.members)
    }

    fn permissions(&self, bot_account: &BotAccountDetailed) -> PermissionGrid {
        PermissionGrid::new(bot_account.bot_account.id, &bot_account.permissions)
    }
}

pub(super) async fn index(
    State(state): State<AppState>,
    User(user): User,
    Query(params): Query<HashMap<Param<BotAccountFilter>, Vec<String>>>,
) -> Result<BotAccountsPage, UiError> {
    let list = ListState::new(params);
    let order_by = list.order_by(SORT_FIELDS)?;
    let limit = list.limit()?;

    let query = BotAccountQuery {
        filter: Some(to_filter(&list)?),
        limit,
        offset: 0,
        order_by: OrderBySet::One(order_by),
    };

    let (Json(bot_accounts), Json(people), layout) = tokio::try_join!(
        index_bot_accounts_detailed(State(state.clone()), user, JsonExtractor(query)),
        index_people(
            State(state.clone()),
            user,
            JsonExtractor(PersonQuery::default())
        ),
        Layout::load(&state, user),
    )?;

    Ok(BotAccountsPage {
        layout,
        list,
        order_by,
        paging: Paging::new(limit, bot_accounts.len()),
        bot_accounts,
        people: person_options(&people),
        staff: staff_options(),
    })
}

#[derive(Deserialize)]
pub(super) struct CreateForm {
    description: String,
}

pub(super) async fn create(
    State(state): State<AppState>,
    User(user): User,
    headers: HeaderMap,
    Form(CreateForm { description }): Form<CreateForm>,
) -> Result<Redirect, UiError> {
    let bot_account = NewBotAccount {
        record: BotAccountSimpleFields {
            description: NonemptyString::new(description).ok(),
            is_staff: false,
        },
        users: Vec::new(),
        permissions_to_grant: Vec::new(),
    };

    let Json(_) = create_bot_account(State(state), user, JsonExtractor(bot_account)).await?;

    Ok(redirect_back(&headers))
}

#[derive(Deserialize)]
pub(super) struct DescriptionForm {
    description: String,
    is_staff: bool,
}

pub(super) async fn set_description(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(DescriptionForm {
        description,
        is_staff,
    }): Form<DescriptionForm>,
) -> Result<Redirect, UiError> {
    let update = BotAccountUpdate {
        description: NonemptyString::new(description).ok(),
        is_staff,
    };

    let Json(_) = update_bot_account(State(state), user, Path(IdParam { id }), JsonExtractor(update)).await?;

    Ok(redirect_back(&headers))
}

#[derive(Deserialize)]
pub(super) struct StaffForm {
    is_staff: Option<String>,
    description: String,
}

pub(super) async fn set_staff(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(StaffForm {
        is_staff,
        description,
    }): Form<StaffForm>,
) -> Result<Redirect, UiError> {
    let update = BotAccountUpdate {
        description: NonemptyString::new(description).ok(),
        is_staff: is_staff.is_some(),
    };

    let Json(_) = update_bot_account(State(state), user, Path(IdParam { id }), JsonExtractor(update)).await?;

    Ok(redirect_back(&headers))
}

pub(super) async fn update_members(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
    Form(MembersForm { add, remove }): Form<MembersForm>,
) -> Result<Redirect, UiError> {
    if let Some(person) = add {
        let Json(_) = add_people_to_bot_account(
            State(state.clone()),
            user,
            Path(IdParam { id }),
            JsonExtractor(vec![person]),
        )
        .await?;
    }

    if let Some(person) = remove {
        let Json(_) = remove_people_from_bot_account(
            State(state),
            user,
            Path(IdParam { id }),
            JsonExtractor(vec![person]),
        )
        .await?;
    }

    Ok(redirect_back(&headers))
}

pub(super) async fn delete(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    headers: HeaderMap,
) -> Result<Redirect, UiError> {
    let Json(_) = delete_resource::<BotAccountSimpleFields>(State(state), user, Path(IdParam { id })).await?;

    Ok(redirect_back(&headers))
}
