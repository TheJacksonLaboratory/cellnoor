use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(scamplers_schema::people)]
#[allow(non_camel_case_types)]
pub enum PersonOrderBy {
    email {
        #[serde(default)]
        descending: bool,
    },
    id {
        #[serde(default)]
        descending: bool,
    },
    name {
        #[serde(default)]
        descending: bool,
    },
}

impl Default for PersonOrderBy {
    fn default() -> Self {
        Self::name { descending: false }
    }
}

#[filter]
pub struct PersonFilter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
    emails: Option<Vec<String>>,
    institution_ids: Option<Vec<Uuid>>,
    orcids: Option<Vec<String>>,
    microsoft_entra_oids: Option<Vec<Uuid>>,
}

impl PersonFilter {
    #[must_use]
    pub fn ids(&self) -> Option<&[Uuid]> {
        self.ids.as_deref()
    }

    #[must_use]
    pub fn names(&self) -> Option<&[String]> {
        self.names.as_deref()
    }

    #[must_use]
    pub fn emails(&self) -> Option<&[String]> {
        self.emails.as_deref()
    }

    #[must_use]
    pub fn institution_ids(&self) -> Option<&[Uuid]> {
        self.institution_ids.as_deref()
    }

    #[must_use]
    pub fn orcids(&self) -> Option<&[String]> {
        self.orcids.as_deref()
    }

    #[must_use]
    pub fn microsoft_entra_oids(&self) -> Option<&[Uuid]> {
        self.microsoft_entra_oids.as_deref()
    }
}

#[cfg(feature = "app")]
pub type PersonQuery = crate::generic_query::Query<PersonFilter, PersonOrderBy>;

#[cfg(feature = "app")]
impl PersonQuery {
    pub fn set_institution_id(&mut self, institution_id: Uuid) {
        let Some(filter) = &mut self.filter else {
            self.filter = Some(PersonFilter {
                institution_ids: Some(vec![institution_id]),
                ..Default::default()
            });

            return;
        };

        filter.institution_ids.replace(vec![institution_id]);
    }
}

uuid_newtype!(PersonId, "/{id}");
