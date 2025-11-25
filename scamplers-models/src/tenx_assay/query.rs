use macro_attributes::filter;
use uuid::Uuid;

use crate::tenx_assay::common::{LibraryType, SampleMultiplexing};

#[filter]
pub struct TenxAssayFilter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
    library_types: Option<Vec<Vec<LibraryType>>>,
    sample_multiplexing: Option<Vec<SampleMultiplexing>>,
    chemistry_versions: Option<Vec<String>>,
    chromium_chips: Option<Vec<String>>,
    #[serde(skip)]
    library_types_are_sorted: bool,
}

impl TenxAssayFilter {
    #[must_use]
    pub fn ids(&self) -> Option<&[Uuid]> {
        self.ids.as_deref()
    }

    #[must_use]
    pub fn names(&self) -> Option<&[String]> {
        self.names.as_deref()
    }

    pub fn sorted_library_types(&mut self) -> Option<&[Vec<LibraryType>]> {
        if self.library_types_are_sorted {
            return self.library_types.as_deref();
        }

        let Some(library_types) = &mut self.library_types else {
            self.library_types_are_sorted = true;
            return None;
        };

        for library_type_group in &mut *library_types {
            library_type_group.sort();
        }
        self.library_types_are_sorted = true;

        Some(&*library_types)
    }

    #[must_use]
    pub fn sample_multiplexing(&self) -> Option<&[SampleMultiplexing]> {
        self.sample_multiplexing.as_deref()
    }

    #[must_use]
    pub fn chemistry_versions(&self) -> Option<&[String]> {
        self.chemistry_versions.as_deref()
    }

    #[must_use]
    pub fn chromium_chips(&self) -> Option<&[String]> {
        self.chromium_chips.as_deref()
    }
}
