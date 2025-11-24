#![allow(dead_code)]

use clap::Parser;
use scamplers_models::{institution, lab, person};
use ts_rs::TS;

#[derive(clap::Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, default_value = "scamplers-types")]
    output_dir: String,
}

#[derive(TS)]
#[ts(optional_fields)]
struct Query<F, O>
where
    F: TS,
    O: TS,
    <O as TS>::OptionInnerType: TS,
{
    #[ts(inline)]
    filter: Option<F>,
    limit: Option<i64>,
    offset: Option<i64>,
    #[ts(inline)]
    order_by: Option<Vec<O>>,
}

#[derive(TS)]
struct InstitutionQuery(
    #[ts(inline)] Query<institution::InstitutionFilter, institution::InstitutionOrderBy>,
);

#[derive(TS)]
struct PersonQuery(#[ts(inline)] Query<person::PersonFilter, person::PersonOrderBy>);

#[derive(TS)]
struct LabQuery(#[ts(inline)] Query<lab::LabFilter, lab::LabOrderBy>);

fn main() {
    let Cli { output_dir } = Cli::parse();

    institution::InstitutionCreation::export_all_to(&output_dir).unwrap();
    InstitutionQuery::export_all_to(&output_dir).unwrap();
    institution::Institution::export_all_to(&output_dir).unwrap();

    person::PersonCreation::export_all_to(&output_dir).unwrap();
    PersonQuery::export_all_to(&output_dir).unwrap();
    person::Person::export_all_to(&output_dir).unwrap();

    lab::LabCreation::export_all_to(&output_dir).unwrap();
    LabQuery::export_all_to(&output_dir).unwrap();
    lab::Lab::export_all_to(&output_dir).unwrap();
}
