#![allow(dead_code)]

use clap::Parser;
use scamplers_api::api::ErrorResponse;
use scamplers_models::{
    institution::{Institution, InstitutionCreation, InstitutionFilter, InstitutionOrderBy},
    lab::{Lab, LabCreation, LabFilter, LabOrderBy},
    person::{Person, PersonCreation, PersonFilter, PersonOrderBy},
    specimen::{Specimen, SpecimenCreation, SpecimenFilter, SpecimenOrderBy},
    suspension::{
        Suspension, SuspensionCreation, SuspensionFilter, SuspensionOrderBy, SuspensionSummary,
        measurement::{
            CellSuspensionMeasurementCreation, NucleusSuspensionMeasurementCreation,
            SuspensionMeasurement,
        },
    },
    suspension_pool::{
        SuspensionPool, SuspensionPoolCreation, SuspensionPoolFilter, SuspensionPoolOrderBy,
        measurement::{
            CellSuspensionPoolMeasurementCreation, NucleusSuspensionPoolMeasurementCreation,
            SuspensionPoolMeasurement,
        },
    },
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

#[derive(TS)]
struct SuspensionQuery(#[ts(inline)] Query<SuspensionFilter, SuspensionOrderBy>);

#[derive(TS)]
struct SuspensionPoolQuery(#[ts(inline)] Query<SuspensionPoolFilter, SuspensionPoolOrderBy>);

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

    SuspensionCreation::export_all_to(&output_dir).unwrap();
    SuspensionQuery::export_all_to(&output_dir).unwrap();
    SuspensionSummary::export_all_to(&output_dir).unwrap();
    Suspension::export_all_to(&output_dir).unwrap();
    CellSuspensionMeasurementCreation::export_all_to(&output_dir).unwrap();
    NucleusSuspensionMeasurementCreation::export_all_to(&output_dir).unwrap();
    SuspensionMeasurement::export_all_to(&output_dir).unwrap();

    SuspensionPoolCreation::export_all_to(&output_dir).unwrap();
    SuspensionPool::export_all_to(&output_dir).unwrap();
    SuspensionPoolQuery::export_all_to(&output_dir).unwrap();
    CellSuspensionPoolMeasurementCreation::export_all_to(&output_dir).unwrap();
    NucleusSuspensionPoolMeasurementCreation::export_all_to(&output_dir).unwrap();
    SuspensionPoolMeasurement::export_all_to(&output_dir).unwrap();

    ErrorResponse::export_all_to(&output_dir).unwrap();
}
