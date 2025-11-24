#![allow(dead_code)]

use clap::Parser;
use scamplers_models::{
    institution::{Institution, InstitutionCreation, InstitutionFilter, InstitutionOrderBy},
    lab::{Lab, LabCreation, LabFilter, LabOrderBy},
    person::{Person, PersonCreation, PersonFilter, PersonOrderBy},
    specimen::{Specimen, SpecimenCreation, SpecimenFilter, SpecimenOrderBy},
};
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
struct InstitutionQuery(#[ts(inline)] Query<InstitutionFilter, InstitutionOrderBy>);

#[derive(TS)]
struct PersonQuery(#[ts(inline)] Query<PersonFilter, PersonOrderBy>);

#[derive(TS)]
struct LabQuery(#[ts(inline)] Query<LabFilter, LabOrderBy>);

#[derive(TS)]
struct SpecimenQuery(#[ts(inline)] Query<SpecimenFilter, SpecimenOrderBy>);

fn main() {
    let Cli { output_dir } = Cli::parse();

    InstitutionCreation::export_all_to(&output_dir).unwrap();
    InstitutionQuery::export_all_to(&output_dir).unwrap();
    Institution::export_all_to(&output_dir).unwrap();

    PersonCreation::export_all_to(&output_dir).unwrap();
    PersonQuery::export_all_to(&output_dir).unwrap();
    Person::export_all_to(&output_dir).unwrap();

    LabCreation::export_all_to(&output_dir).unwrap();
    LabQuery::export_all_to(&output_dir).unwrap();
    Lab::export_all_to(&output_dir).unwrap();

    SpecimenCreation::export_all_to(&output_dir).unwrap();
    SpecimenQuery::export_all_to(&output_dir).unwrap();
    Specimen::export_all_to(&output_dir).unwrap();
}
