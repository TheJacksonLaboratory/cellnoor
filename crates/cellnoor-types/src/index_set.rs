use macro_attributes::base_model;
use nonempty::NonemptyString;

use crate::index_set::{
    dual_index_set_record::DualIndexSetRecord, single_index_set_record::SingleIndexSetRecord,
};

mod dual_index_set_record {
    use macro_attributes::select;
    use nonempty::NonemptyString;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "dual_index_set"))]
    pub struct DualIndexSetRecord {
        pub name: NonemptyString,
        pub kit: NonemptyString,
        pub well: NonemptyString,
        pub index_i7: NonemptyString,
        pub index2_workflow_a_i5: NonemptyString,
        pub index2_workflow_b_i5: NonemptyString,
    }
}

pub type DualIndexSet = DualIndexSetRecord;

#[base_model]
pub struct NewDualIndexSet {
    #[cfg_attr(feature = "serde", serde(alias = "index(i7)"))]
    index_i7: String,
    #[cfg_attr(feature = "serde", serde(alias = "index2_workflow_a(i5)"))]
    index2_workflow_a_i5: String,
    #[cfg_attr(feature = "serde", serde(alias = "index2_workflow_b(i5)"))]
    index2_workflow_b_i5: String,
}

mod single_index_set_record {
    use macro_attributes::select;
    use nonempty::NonemptyString;

    #[select]
    pub struct SingleIndexSetRecord {
        name: NonemptyString,
        kit: NonemptyString,
        well: NonemptyString,
        sequences: Vec<NonemptyString>,
    }
}

#[base_model]
pub struct NewSingleIndexSet(String, [NonemptyString; 4]);

pub type SingleIndexSet = SingleIndexSetRecord;
