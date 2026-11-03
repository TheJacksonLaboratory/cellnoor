use std::{collections::HashMap, fmt, hash::Hash, str::FromStr};

use cellnoor_types::{order_by::OrderBy, query::OrderField};
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use serde::{Deserialize, Deserializer, de};
use uuid::Uuid;

use crate::ui::UiError;

pub(super) const PAGE_SIZES: [i64; 5] = [50, 100, 250, 500, 1000];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub(super) enum ListParam {
    Sort,
    Desc,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Param<F> {
    List(ListParam),
    Value(F),
    Add(F),
    Remove(F),
    Search(F),
}

impl<F: fmt::Display> fmt::Display for Param<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List(param) => param.fmt(f),
            Self::Value(field) => field.fmt(f),
            Self::Add(field) => write!(f, "add.{field}"),
            Self::Remove(field) => write!(f, "remove.{field}"),
            Self::Search(field) => write!(f, "search.{field}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown parameter `{0}`")]
pub(super) struct UnknownParam(String);

impl<F: FromStr> FromStr for Param<F> {
    type Err = UnknownParam;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(param) = s.parse() {
            return Ok(Self::List(param));
        }

        let field = |name: &str| name.parse().map_err(|_| UnknownParam(s.to_owned()));

        match s.split_once('.') {
            Some(("add", name)) => field(name).map(Self::Add),
            Some(("remove", name)) => field(name).map(Self::Remove),
            Some(("search", name)) => field(name).map(Self::Search),
            _ => field(s).map(Self::Value),
        }
    }
}

impl<'de, F: FromStr> Deserialize<'de> for Param<F> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub(super) struct ListState<F>(HashMap<Param<F>, Vec<String>>);

impl<F: Copy + Eq + Hash> ListState<F> {
    pub(super) fn new(params: HashMap<Param<F>, Vec<String>>) -> Self {
        Self(params)
    }

    fn values(&self, param: Param<F>) -> impl Iterator<Item = &str> {
        self.0
            .get(&param)
            .into_iter()
            .flatten()
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    pub(super) fn last(&self, param: Param<F>) -> Option<&str> {
        self.values(param).last()
    }

    pub(super) fn value(&self, field: F) -> &str {
        self.last(Param::Value(field)).unwrap_or_default()
    }

    pub(super) fn selected(&self, field: F) -> Vec<&str> {
        let removed: Vec<_> = self.values(Param::Remove(field)).collect();
        let mut selected = Vec::new();

        for value in self
            .values(Param::Value(field))
            .chain(self.values(Param::Add(field)))
        {
            if !removed.contains(&value) && !selected.contains(&value) {
                selected.push(value);
            }
        }

        selected
    }

    pub(super) fn tags(&self, field: F) -> Vec<Opt> {
        self.selected(field)
            .into_iter()
            .map(|value| Opt::new(value, value))
            .collect()
    }

    pub(super) fn picker(&self, field: F, options: &[Opt]) -> Picker<Param<F>> {
        let selected = self.selected(field);
        let picked =
            self.0.contains_key(&Param::Add(field)) || self.0.contains_key(&Param::Remove(field));
        // Picking an option ends the search, so that every option shows again
        let search = if picked {
            ""
        } else {
            self.last(Param::Search(field)).unwrap_or_default()
        };

        Picker {
            value_name: Some(Param::Value(field)),
            add_name: Param::Add(field),
            remove_name: Param::Remove(field),
            search_name: Some(Param::Search(field)),
            search: search.to_owned(),
            open: picked || !search.is_empty(),
            chips: chips(options, &selected),
            items: items(options, &selected, search),
        }
    }

    pub(super) fn limit(&self) -> Result<Option<i64>, UiError> {
        match self.last(Param::List(ListParam::Limit)) {
            None => Ok(Some(PAGE_SIZES[0])),
            Some("all") => Ok(None),
            Some(limit) => limit
                .parse::<u32>()
                .map(|limit| Some(limit.into()))
                .map_err(|_| UiError::BadRequest(format!("invalid limit `{limit}`"))),
        }
    }

    pub(super) fn order_by<O: OrderField>(
        &self,
        offered: &[(O, &str)],
    ) -> Result<OrderBy<O>, UiError> {
        let field = match self.last(Param::List(ListParam::Sort)) {
            None => O::default_field(),
            Some(sort) => offered
                .iter()
                .find(|(field, _)| field.as_ref() == sort)
                .map(|(field, _)| *field)
                .ok_or_else(|| UiError::BadRequest(format!("cannot sort by `{sort}`")))?,
        };

        let desc = match self.last(Param::List(ListParam::Desc)) {
            None => O::default_desc(),
            Some(desc) => desc
                .parse()
                .map_err(|_| UiError::BadRequest(format!("invalid sort direction `{desc}`")))?,
        };

        Ok(OrderBy { field, desc })
    }
}

pub(super) struct Paging {
    pub(super) limit: Option<i64>,
    pub(super) next: Option<i64>,
}

impl Paging {
    pub(super) fn new(limit: Option<i64>, count: usize) -> Self {
        let is_full = |limit: &i64| usize::try_from(*limit).is_ok_and(|limit| count >= limit);

        Self {
            limit,
            next: limit.filter(is_full).map(|limit| limit + PAGE_SIZES[0]),
        }
    }

    pub(super) fn custom(&self) -> Option<i64> {
        self.limit.filter(|limit| !PAGE_SIZES.contains(limit))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Opt {
    pub(super) value: String,
    pub(super) label: String,
}

impl Opt {
    pub(super) fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub(super) trait Labels {
    fn label_of(&self, value: impl ToString) -> &str;
}

impl Labels for [Opt] {
    fn label_of(&self, value: impl ToString) -> &str {
        let value = value.to_string();

        self.iter()
            .find(|option| option.value == value)
            .map_or("", |option| option.label.as_str())
    }
}

pub(super) struct Picker<N> {
    pub(super) value_name: Option<N>,
    pub(super) add_name: N,
    pub(super) remove_name: N,
    pub(super) search_name: Option<N>,
    pub(super) search: String,
    pub(super) open: bool,
    pub(super) chips: Vec<Opt>,
    pub(super) items: Vec<PickerItem>,
}

#[derive(Debug, PartialEq)]
pub(super) struct PickerItem {
    pub(super) option: Opt,
    pub(super) selected: bool,
    pub(super) matches: bool,
}

impl<N> Picker<N> {
    pub(super) fn has_matches(&self) -> bool {
        self.items.iter().any(|item| item.matches)
    }
}

impl Picker<&'static str> {
    pub(super) fn members(options: &[Opt], members: &[Uuid]) -> Self {
        let members: Vec<_> = members.iter().map(Uuid::to_string).collect();
        let selected: Vec<_> = members.iter().map(String::as_str).collect();

        Self {
            value_name: None,
            add_name: "add",
            remove_name: "remove",
            search_name: None,
            search: String::new(),
            open: false,
            chips: chips(options, &selected),
            items: items(options, &selected, ""),
        }
    }
}

fn chips(options: &[Opt], selected: &[&str]) -> Vec<Opt> {
    selected
        .iter()
        .map(|&value| {
            options
                .iter()
                .find(|option| option.value == value)
                .cloned()
                .unwrap_or_else(|| Opt::new(value, value))
        })
        .collect()
}

fn items(options: &[Opt], selected: &[&str], search: &str) -> Vec<PickerItem> {
    let search = search.to_lowercase();

    options
        .iter()
        .map(|option| PickerItem {
            option: option.clone(),
            selected: selected.contains(&option.value.as_str()),
            matches: option.label.to_lowercase().contains(&search),
        })
        .collect()
}

pub(super) fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|&value| value.to_owned()).collect()
}

pub(super) fn parse<T: FromStr>(value: &str) -> Result<T, UiError> {
    value
        .parse()
        .map_err(|_| UiError::BadRequest(format!("invalid value `{value}`")))
}

pub(super) fn parse_all<T: FromStr>(values: &[&str]) -> Result<Vec<T>, UiError> {
    values.iter().map(|value| parse(value)).collect()
}

pub(super) fn start_of_day(date: &str) -> Result<Timestamp, UiError> {
    utc(date, 0, 0, 0, 0)
}

pub(super) fn end_of_day(date: &str) -> Result<Timestamp, UiError> {
    utc(date, 23, 59, 59, 999_000_000)
}

fn utc(
    date: &str,
    hour: i8,
    minute: i8,
    second: i8,
    nanosecond: i32,
) -> Result<Timestamp, UiError> {
    date.parse::<Date>()
        .and_then(|day| {
            day.at(hour, minute, second, nanosecond)
                .to_zoned(TimeZone::UTC)
        })
        .map(|zoned| zoned.timestamp())
        .map_err(|error| UiError::BadRequest(format!("invalid date `{date}`: {error}")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::http::Uri;
    use axum_extra::extract::Query;
    use pretty_assertions::assert_eq;

    use super::{ListParam, ListState, Opt, Paging, Param, PickerItem, end_of_day, start_of_day};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    enum Field {
        Species,
        ReceivedFrom,
    }

    fn parse(query: &str) -> Result<ListState<Field>, axum_extra::extract::QueryRejection> {
        let uri: Uri = format!("/?{query}").parse().unwrap();
        let Query(params) = Query::<HashMap<Param<Field>, Vec<String>>>::try_from_uri(&uri)?;

        Ok(ListState::new(params))
    }

    #[test]
    fn repeated_keys_collect_into_one_entry() {
        let list = parse("species=a&sort=name&species=b&add.species=c").unwrap();

        assert_eq!(list.selected(Field::Species), ["a", "b", "c"]);
        assert_eq!(list.last(Param::List(ListParam::Sort)), Some("name"));
    }

    #[test]
    fn unknown_params_are_rejected() {
        assert!(parse("colour=red").is_err());
        assert!(parse("add.colour=red").is_err());
    }

    #[test]
    fn param_names_round_trip() {
        let params = [
            Param::List(ListParam::Limit),
            Param::Value(Field::ReceivedFrom),
            Param::Add(Field::Species),
            Param::Remove(Field::Species),
            Param::Search(Field::Species),
        ];

        for param in params {
            assert_eq!(param.to_string().parse::<Param<Field>>().unwrap(), param);
        }

        assert_eq!(
            Param::Remove(Field::ReceivedFrom).to_string(),
            "remove.received_from"
        );
    }

    #[test]
    fn selection_drops_removed_empty_and_duplicate_values() {
        let list =
            parse("species=a&species=&species=b&species=a&add.species=c&remove.species=b").unwrap();

        assert_eq!(list.selected(Field::Species), ["a", "c"]);
    }

    #[test]
    fn later_values_win() {
        let list = parse("limit=50&desc=true&received_from=&limit=100&desc=false").unwrap();

        assert_eq!(list.limit().unwrap(), Some(100));
        assert_eq!(list.last(Param::List(ListParam::Desc)), Some("false"));
        assert_eq!(list.value(Field::ReceivedFrom), "");
    }

    #[test]
    fn limit_defaults_to_the_smallest_page() {
        assert_eq!(parse("").unwrap().limit().unwrap(), Some(50));
        assert_eq!(parse("limit=all").unwrap().limit().unwrap(), None);
        parse("limit=-1").unwrap().limit().unwrap_err();
    }

    #[test]
    fn dropdown_stays_open_while_being_edited() {
        let options = [Opt::new("a", "Apple"), Opt::new("b", "Banana")];

        let closed = parse("species=a").unwrap().picker(Field::Species, &options);
        assert!(!closed.open);
        assert_eq!(closed.chips, [Opt::new("a", "Apple")]);

        let item = |value, label, selected, matches| PickerItem {
            option: Opt::new(value, label),
            selected,
            matches,
        };

        let searched = parse("search.species=NAN").unwrap();
        let searched = searched.picker(Field::Species, &options);
        assert!(searched.open);
        assert_eq!(
            searched.items,
            [item("a", "Apple", false, false), item("b", "Banana", false, true)]
        );

        let picked = parse("species=a&search.species=NAN&add.species=b").unwrap();
        let picked = picked.picker(Field::Species, &options);
        assert!(picked.open);
        assert_eq!(picked.search, "");
        assert_eq!(
            picked.items,
            [item("a", "Apple", true, true), item("b", "Banana", true, true)]
        );
    }

    #[test]
    fn unknown_values_keep_their_raw_value_as_label() {
        let picker = parse("species=z").unwrap().picker(Field::Species, &[]);

        assert_eq!(picker.chips, [Opt::new("z", "z")]);
    }

    #[test]
    fn load_more_only_when_the_page_is_full() {
        assert_eq!(Paging::new(Some(50), 50).next, Some(100));
        assert_eq!(Paging::new(Some(50), 49).next, None);
        assert_eq!(Paging::new(None, 1000).next, None);
        assert_eq!(Paging::new(Some(150), 0).custom(), Some(150));
        assert_eq!(Paging::new(Some(100), 0).custom(), None);
    }

    #[test]
    fn days_are_utc() {
        assert_eq!(
            start_of_day("2026-01-02").unwrap().to_string(),
            "2026-01-02T00:00:00Z"
        );
        assert_eq!(
            end_of_day("2026-01-02").unwrap().to_string(),
            "2026-01-02T23:59:59.999Z"
        );
        start_of_day("yesterday").unwrap_err();
    }
}
