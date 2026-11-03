use std::collections::{BTreeMap, BTreeSet, HashMap};

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::Query;
use cellnoor_types::{
    cdna::creation::LibraryType,
    chromium_dataset::{
        ChromiumDatasetDetailed, ChromiumDatasetField, ChromiumDatasetParsedFileData,
        ChromiumDatasetPredicate, ChromiumDatasetPredicateInner, ChromiumDatasetQuery,
    },
    file_tree::{FileTreeNode, FileTreeNodeKind},
    filter::Filter,
    operator::{ArrayOperator, SimpleStringOperator, StringOperator, TimestampOperator},
    order_by::{OrderBy, OrderBySet},
    person::PersonQuery,
    project::{ProjectPredicate, ProjectQuery},
    specimen::{
        BlockEmbeddingMatrixOperator, Fixative, FixativeOperator, SavedSpecimenRecord, Species,
        SpeciesOperator, SpecimenPredicate, SpecimenType, SpecimenTypeOperator,
        ThermalPreservationMethod, ThermalPreservationMethodOperator,
        creation::block::BlockEmbeddingMatrix,
    },
    suspension_pool::TaggedSpecimen,
    tenx_assay::{SampleMultiplexing, SampleMultiplexingOperator, TenxAssayPredicate},
};
use jiff::Timestamp;
use serde::Deserialize;
use strum::VariantArray;
use uuid::Uuid;

use crate::{
    extract::JsonExtractor,
    handlers::{
        IdParam,
        chromium_datasets::{index_chromium_datasets_detailed, show_chromium_dataset},
        people::index_people,
        projects::index_projects,
        tenx_assays::index_tenx_assays,
    },
    state::AppState,
    ui::{
        Layout, UiError, User,
        format::{Choice, options, parse_choices, person_options},
        params::{Labels, ListState, Opt, Paging, Param, end_of_day, start_of_day, strings},
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
pub(super) enum DatasetFilter {
    Name,
    Project,
    DeliveredFrom,
    DeliveredTo,
    AssayName,
    AssayMultiplexingType,
    AssayLibraryType,
    SpecimenName,
    SpecimenType,
    SpecimenSpecies,
    SpecimenEmbeddedIn,
    SpecimenFixative,
    SpecimenThermalPreservationMethod,
    SpecimenReceivedFrom,
    SpecimenReceivedTo,
}

const SORT_FIELDS: &[(ChromiumDatasetField, &str)] = &[
    (ChromiumDatasetField::DeliveredAt, "Date delivered"),
    (ChromiumDatasetField::Name, "Name"),
];

const SPECIMEN_LIMIT: usize = 8;

fn to_filter(
    list: &ListState<DatasetFilter>,
) -> Result<Filter<ChromiumDatasetPredicate>, UiError> {
    use ChromiumDatasetPredicateInner as Dataset;
    use DatasetFilter as F;

    let mut filters = Vec::new();

    for &field in F::VARIANTS {
        let values = list.selected(field);
        let date = list.last(Param::Value(field));

        let filter: Option<Filter<ChromiumDatasetPredicate>> = match field {
            F::Name => Some(Dataset::Name(StringOperator::TrgmAnyUnlessEmpty(strings(&values))).into()),
            F::Project => Some(
                ProjectPredicate::Name(SimpleStringOperator::InUnlessEmpty(strings(&values)).into())
                    .into(),
            ),
            F::DeliveredFrom => date
                .map(start_of_day)
                .transpose()?
                .map(|from| Dataset::DeliveredAt(TimestampOperator::Gte(from)).into()),
            F::DeliveredTo => date
                .map(end_of_day)
                .transpose()?
                .map(|to| Dataset::DeliveredAt(TimestampOperator::Lte(to)).into()),
            F::AssayName => Some(
                TenxAssayPredicate::Name(SimpleStringOperator::InUnlessEmpty(strings(&values)).into())
                    .into(),
            ),
            F::AssayMultiplexingType => Some(
                TenxAssayPredicate::SampleMultiplexing(SampleMultiplexingOperator::InUnlessEmpty(
                    parse_choices(&values)?,
                ))
                .into(),
            ),
            F::AssayLibraryType => Some(
                TenxAssayPredicate::LibraryTypes(ArrayOperator::Contains(parse_choices(&values)?))
                    .into(),
            ),
            F::SpecimenName => Some(
                SpecimenPredicate::Name(StringOperator::TrgmAnyUnlessEmpty(strings(&values))).into(),
            ),
            F::SpecimenType => Some(
                SpecimenPredicate::Type(SpecimenTypeOperator::InUnlessEmpty(parse_choices(&values)?))
                    .into(),
            ),
            F::SpecimenSpecies => Some(
                SpecimenPredicate::Species(SpeciesOperator::InUnlessEmpty(parse_choices(&values)?))
                    .into(),
            ),
            F::SpecimenEmbeddedIn => Some(
                SpecimenPredicate::EmbeddedIn(BlockEmbeddingMatrixOperator::InUnlessEmpty(
                    parse_choices(&values)?,
                ))
                .into(),
            ),
            F::SpecimenFixative => Some(
                SpecimenPredicate::Fixative(FixativeOperator::InUnlessEmpty(parse_choices(&values)?))
                    .into(),
            ),
            F::SpecimenThermalPreservationMethod => Some(
                SpecimenPredicate::ThermalPreservationMethod(
                    ThermalPreservationMethodOperator::InUnlessEmpty(parse_choices(&values)?),
                )
                .into(),
            ),
            F::SpecimenReceivedFrom => date
                .map(start_of_day)
                .transpose()?
                .map(|from| SpecimenPredicate::ReceivedAt(TimestampOperator::Gte(from)).into()),
            F::SpecimenReceivedTo => date
                .map(end_of_day)
                .transpose()?
                .map(|to| SpecimenPredicate::ReceivedAt(TimestampOperator::Lte(to)).into()),
        };

        filters.extend(filter);
    }

    Ok(Filter::AllOf(filters))
}

#[derive(Template, WebTemplate)]
#[template(path = "chromium_datasets.html")]
pub(super) struct DatasetsPage {
    layout: Layout,
    list: ListState<DatasetFilter>,
    order_by: OrderBy<ChromiumDatasetField>,
    paging: Paging,
    datasets: Vec<ChromiumDatasetDetailed>,
    projects: Vec<Opt>,
    assays: Vec<Opt>,
    multiplexing_types: Vec<Opt>,
    library_types: Vec<Opt>,
    specimen_types: Vec<Opt>,
    species: Vec<Opt>,
    embedding_matrices: Vec<Opt>,
    fixatives: Vec<Opt>,
    thermal_preservation_methods: Vec<Opt>,
}

pub(super) async fn index(
    State(state): State<AppState>,
    User(user): User,
    Query(params): Query<HashMap<Param<DatasetFilter>, Vec<String>>>,
) -> Result<DatasetsPage, UiError> {
    let list = ListState::new(params);
    let order_by = list.order_by(SORT_FIELDS)?;
    let limit = list.limit()?;

    let query = ChromiumDatasetQuery {
        filter: Some(to_filter(&list)?),
        limit,
        offset: 0,
        order_by: OrderBySet::One(order_by),
    };

    let (Json(datasets), Json(projects), Json(assays), layout) = tokio::try_join!(
        index_chromium_datasets_detailed(State(state.clone()), user, JsonExtractor(query)),
        index_projects(
            State(state.clone()),
            user,
            JsonExtractor(ProjectQuery::default())
        ),
        index_tenx_assays(State(state.clone()), user),
        Layout::load(&state, user),
    )?;

    let mut assay_names: Vec<String> = Vec::new();
    for assay in assays {
        let name = assay.name.to_string();
        if !assay_names.contains(&name) {
            assay_names.push(name);
        }
    }

    Ok(DatasetsPage {
        layout,
        list,
        order_by,
        paging: Paging::new(limit, datasets.len()),
        datasets,
        projects: projects
            .iter()
            .map(|project| Opt::new(project.record.name.to_string(), project.record.name.to_string()))
            .collect(),
        assays: assay_names.iter().map(|name| Opt::new(name, name)).collect(),
        multiplexing_types: options::<SampleMultiplexing>(),
        library_types: options::<LibraryType>(),
        specimen_types: options::<SpecimenType>(),
        species: options::<Species>(),
        embedding_matrices: options::<BlockEmbeddingMatrix>(),
        fixatives: options::<Fixative>(),
        thermal_preservation_methods: options::<ThermalPreservationMethod>(),
    })
}

#[derive(Deserialize)]
pub(super) struct Selection {
    file: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "chromium_dataset.html")]
pub(super) struct DatasetPage {
    layout: Layout,
    dataset: ChromiumDatasetDetailed,
    file: Option<String>,
    specimens: SpecimenTable,
}

pub(super) async fn show(
    State(state): State<AppState>,
    User(user): User,
    Path(IdParam { id }): Path<IdParam>,
    Query(Selection { file }): Query<Selection>,
) -> Result<DatasetPage, UiError> {
    let (Json(dataset), Json(people), layout) = tokio::try_join!(
        show_chromium_dataset(State(state.clone()), user, Path(IdParam { id })),
        index_people(
            State(state.clone()),
            user,
            JsonExtractor(PersonQuery::default())
        ),
        Layout::load(&state, user),
    )?;

    Ok(DatasetPage {
        layout,
        specimens: SpecimenTable::new(&dataset.specimens, &person_options(&people)),
        dataset,
        file,
    })
}

type Node = FileTreeNode<ChromiumDatasetParsedFileData>;

impl DatasetPage {
    fn files(&self) -> FileTree<'_> {
        FileTree {
            nodes: &self.dataset.files,
            selected: self.file.as_deref(),
        }
    }

    fn selected_file(&self) -> Option<SelectedFile<'_>> {
        find_file(&self.dataset.files, self.file.as_deref()?)
    }

    fn library_types(&self) -> String {
        let library_types = self.dataset.assay.library_types.iter().flatten();

        library_types
            .map(Choice::label)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn multiplexing(&self) -> String {
        self.dataset
            .assay
            .sample_multiplexing
            .map(|multiplexing| multiplexing.label())
            .unwrap_or_default()
    }

    fn ilab_request_ids(&self) -> Option<String> {
        let additional_data = self.dataset.chromium_run.record.additional_data.as_ref()?;
        let ids = additional_data.get("ilab_request_ids")?.as_array()?;

        let ids: Vec<_> = ids
            .iter()
            .map(|id| id.as_str().map_or_else(|| id.to_string(), str::to_owned))
            .collect();

        Some(ids.join(" | "))
    }
}

#[derive(Template)]
#[template(path = "file_tree.html")]
struct FileTree<'a> {
    nodes: &'a [Node],
    selected: Option<&'a str>,
}

impl<'a> FileTree<'a> {
    fn subtree(&self, nodes: &'a [Node]) -> Self {
        Self {
            nodes,
            selected: self.selected,
        }
    }

    fn contains_selected(&self, directory: &str) -> bool {
        self.selected
            .and_then(|selected| selected.strip_prefix(directory))
            .is_some_and(|rest| rest.starts_with('/'))
    }

    fn is_selected(&self, file: &str) -> bool {
        self.selected == Some(file)
    }
}

struct SelectedFile<'a> {
    name: &'a str,
    raw_file_link: &'a str,
    contents: Contents,
}

enum Contents {
    Raw,
    Json(String),
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

fn find_file<'a>(nodes: &'a [Node], path: &str) -> Option<SelectedFile<'a>> {
    nodes.iter().find_map(|node| match &node.kind {
        FileTreeNodeKind::Directory { children } => find_file(children, path),
        FileTreeNodeKind::File {
            raw_file_link,
            parsed_contents,
        } => (node.path == path).then(|| SelectedFile {
            name: &node.name,
            raw_file_link,
            contents: Contents::new(parsed_contents.as_ref()),
        }),
    })
}

impl Contents {
    fn new(data: Option<&ChromiumDatasetParsedFileData>) -> Self {
        let rows: Vec<_> = match data {
            None => return Self::Raw,
            Some(ChromiumDatasetParsedFileData::Json { data }) => {
                let sorted: BTreeMap<_, _> = data.iter().collect();
                return Self::Json(serde_json::to_string_pretty(&sorted).unwrap_or_default());
            }
            Some(ChromiumDatasetParsedFileData::SingleRowCsv { data }) => vec![data],
            Some(ChromiumDatasetParsedFileData::MultiRowCsv { data }) => data.iter().collect(),
        };

        let columns: BTreeSet<_> = rows.iter().flat_map(|row| row.keys()).collect();

        Self::Table {
            rows: rows
                .iter()
                .map(|row| {
                    columns
                        .iter()
                        .map(|&column| row.get(column).map(ToString::to_string).unwrap_or_default())
                        .collect()
                })
                .collect(),
            columns: columns.into_iter().cloned().collect(),
        }
    }
}

type SpecimenText = fn(&SavedSpecimenRecord, &[Opt]) -> Option<String>;

const SPECIMEN_FIELDS: [(&str, SpecimenText); 8] = [
    ("Name", |s, _| Some(s.name.to_string())),
    ("Species", |s, _| Some(s.species.label())),
    ("Submitted by", |s, people| {
        Some(people.label_of(s.submitted_by).to_owned())
    }),
    ("Type", |s, _| Some(s.type_.label())),
    ("Tissue", |s, _| Some(s.tissue.to_string())),
    ("Preservation", |s, _| {
        s.thermal_preservation_method.map(|method| method.label())
    }),
    ("Embedding matrix", |s, _| {
        s.embedded_in.map(|matrix| matrix.label())
    }),
    ("Fixative", |s, _| s.fixative.map(|fixative| fixative.label())),
];

struct SpecimenTable {
    shared: Vec<(&'static str, String)>,
    columns: Vec<&'static str>,
    rows: Vec<SpecimenRow>,
}

struct SpecimenRow {
    id: Uuid,
    readable_id: String,
    cells: Vec<String>,
    received_at: Timestamp,
}

impl SpecimenTable {
    fn new(specimens: &[TaggedSpecimen], people: &[Opt]) -> Self {
        let records: Vec<_> = specimens.iter().map(|s| &s.specimen.record).collect();

        let (shared, varying): (Vec<_>, Vec<_>) = SPECIMEN_FIELDS
            .iter()
            .map(|&(label, text)| {
                let texts: Vec<_> = records.iter().map(|record| text(record, people)).collect();
                (label, texts)
            })
            .partition(|(_, texts)| texts.first().is_some_and(|first| texts.iter().all(|t| t == first)));

        let rows = records
            .iter()
            .enumerate()
            .map(|(i, record)| SpecimenRow {
                id: *record.id,
                readable_id: record.readable_id.to_string(),
                cells: varying
                    .iter()
                    .map(|(_, texts)| texts[i].clone().unwrap_or_default())
                    .collect(),
                received_at: record.received_at,
            })
            .collect();

        Self {
            shared: shared
                .into_iter()
                .filter_map(|(label, mut texts)| {
                    texts.swap_remove(0).filter(|text| !text.is_empty()).map(|text| (label, text))
                })
                .collect(),
            columns: varying.iter().map(|&(label, _)| label).collect(),
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::http::Uri;
    use axum_extra::extract::Query;
    use cellnoor_types::{
        chromium_dataset::{ChromiumDatasetParsedFileData, ChromiumDatasetPredicateInner},
        file_tree::{FileTreeNode, FileTreeNodeKind},
        filter::Filter,
        operator::{SimpleStringOperator, StringOperator, TimestampOperator},
        project::ProjectPredicate,
        specimen::{SpeciesOperator, SpecimenPredicate, Species},
    };
    use pretty_assertions::assert_eq;

    use super::{DatasetFilter, FileTree, Param, find_file, to_filter};
    use crate::ui::params::ListState;

    fn list(query: &str) -> ListState<DatasetFilter> {
        let uri: Uri = format!("/?{query}").parse().unwrap();
        let Query(params) =
            Query::<HashMap<Param<DatasetFilter>, Vec<String>>>::try_from_uri(&uri).unwrap();

        ListState::new(params)
    }

    #[test]
    fn filters_mirror_the_form() {
        let Filter::AllOf(filters) = to_filter(&list(
            "project=Atlas&specimen_species=homo_sapiens&delivered_to=2026-01-02",
        ))
        .unwrap() else {
            panic!("expected all_of")
        };

        assert!(filters.contains(
            &ProjectPredicate::Name(SimpleStringOperator::InUnlessEmpty(vec!["Atlas".to_owned()]).into())
                .into()
        ));
        assert!(filters.contains(
            &SpecimenPredicate::Species(SpeciesOperator::InUnlessEmpty(vec![Species::HomoSapiens]))
                .into()
        ));
        assert!(filters.contains(
            &ChromiumDatasetPredicateInner::DeliveredAt(TimestampOperator::Lte(
                "2026-01-02T23:59:59.999Z".parse().unwrap()
            ))
            .into()
        ));
        assert!(filters.contains(
            &ChromiumDatasetPredicateInner::Name(StringOperator::TrgmAnyUnlessEmpty(Vec::new()))
                .into()
        ));

        // Every multi-valued field always filters (`*_unless_empty`), but dates only when given
        assert_eq!(filters.len(), 12);
    }

    #[test]
    fn invalid_values_are_rejected() {
        to_filter(&list("specimen_species=dragon")).unwrap_err();
        to_filter(&list("delivered_from=soon")).unwrap_err();
    }

    fn file(path: &str, parsed_contents: Option<ChromiumDatasetParsedFileData>) -> FileTreeNode<ChromiumDatasetParsedFileData> {
        FileTreeNode {
            path: path.to_owned(),
            name: path.rsplit('/').next().unwrap().to_owned(),
            kind: FileTreeNodeKind::File {
                raw_file_link: format!("/files/{path}"),
                parsed_contents,
            },
        }
    }

    #[test]
    fn selected_file_opens_its_directories() {
        let nodes = [
            FileTreeNode {
                path: "per_sample_outs".to_owned(),
                name: "per_sample_outs".to_owned(),
                kind: FileTreeNodeKind::Directory {
                    children: vec![file("per_sample_outs/a/web_summary.html", None)],
                },
            },
            file("per_sample_outs_report.html", None),
        ];

        let tree = FileTree {
            nodes: &nodes,
            selected: Some("per_sample_outs/a/web_summary.html"),
        };

        assert!(tree.contains_selected("per_sample_outs"));
        assert!(!tree.contains_selected("per_sample"));

        let found = find_file(&nodes, "per_sample_outs/a/web_summary.html").unwrap();
        assert_eq!(found.raw_file_link, "/files/per_sample_outs/a/web_summary.html");
        assert!(find_file(&nodes, "per_sample_outs").is_none());
    }
}
