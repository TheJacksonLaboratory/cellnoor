use macro_attributes::select;
pub use query::{ReferencePackageField, ReferencePackagePredicate};

use crate::{Relation, nonempty::NonemptyString};

mod query;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "reference_package"))]
pub struct ReferencePackage {
    pub package_name: NonemptyString,
    pub genome_assembly: NonemptyString,
    pub tenx_reference_version: Option<NonemptyString>,
    pub assembly_source: NonemptyString,
    pub assembly_release: NonemptyString,
    pub annotation_source: NonemptyString,
    pub annotation_release: NonemptyString,
}

impl Relation for ReferencePackage {
    const NAME: &'static str = "reference_package";
}
