#![allow(uncommon_codepoints)]
#![allow(clippy::derivable_impls)]
pub use simple_links::SimpleLinks;

pub mod api_key;
pub mod cdna;
pub mod chromium_dataset;
pub mod chromium_run;
pub mod id;
pub mod index_set;
pub mod institution;
pub mod library;
pub mod multiplexing_tag;
pub mod nucleic_acid_measurement;
pub mod person;
pub mod project;
pub mod query;
pub mod service;
pub(crate) mod simple_links;
pub mod specimen;
pub mod suspension;
pub mod suspension_pool;
pub mod tenx_assay;
pub mod units;

pub mod operator {
    pub use crate::query::filter::{
        BoolOperator, F32Operator, I32Operator, I64Operator, JsonOperator, SimpleStringOperator,
        StringOperator, TimestampOperator, UuidOperator,
    };
}

pub mod filter {
    pub use crate::query::filter::Filter;
}

pub mod order_by {
    pub use crate::query::order_by::{OrderBy, OrderBySet};
}
