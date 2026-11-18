use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::StringOperator,
    query::{Field, OrderField},
    reference_package::ReferencePackage,
};

#[predicate_enum(ReferencePackageField)]
pub enum ReferencePackagePredicate {
    PackageName(StringOperator),
    GenomeAssembly(StringOperator),
    TenxReferenceVersion(StringOperator),
    AssemblySource(StringOperator),
    AssemblyRelease(StringOperator),
    AnnotationSource(StringOperator),
    AnnotationRelease(StringOperator),
}

impl Field for ReferencePackageField {
    const RELATION: &'static str = <ReferencePackage as Relation>::NAME;
}

impl OrderField for ReferencePackageField {
    fn default_field() -> Self {
        Self::PackageName
    }

    fn default_desc() -> bool {
        false
    }
}
