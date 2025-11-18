use clap::Parser;
use scamplers_models::{institution, lab, person};
use ts_rs::TS;

#[derive(clap::Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, default_value = "scamplers-types")]
    output_dir: String,
}

fn main() {
    let Cli { output_dir } = Cli::parse();

    institution::Creation::export_all_to(&output_dir).unwrap();
    institution::Query::export_all_to(&output_dir).unwrap();
    institution::Institution::export_all_to(&output_dir).unwrap();
    person::Creation::export_all_to(&output_dir).unwrap();
    person::Query::export_all_to(&output_dir).unwrap();
    person::Person::export_all_to(&output_dir).unwrap();
    lab::Creation::export_all_to(&output_dir).unwrap();
    lab::Query::export_all_to(&output_dir).unwrap();
    lab::Lab::export_all_to(&output_dir).unwrap();
}
