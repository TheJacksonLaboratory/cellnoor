# **Design**

This document explains only the design of the database schema and Rust crates which make up the RESTful API, which are the most complex items in this repository.

## Database

The database models models a scientific service that takes "specimens", which are all part of "projects", and transforms them through a chain of steps into a "Chromium dataset", which is a single-cell RNA sequencing dataset generated with the [10x Genomics Chromium platform](https://www.10xgenomics.com/platforms/chromium) and NGS. In the future, more 10x Genomics' platforms will be modeled, as well as non-sequencing data.

The main philosophies the database adheres to are:

1. _Row-level security_ - unless a user is staff, they can only see projects they are explicitly a part of. This row-level security applies to all entities that are descendants of projects, such as specimens, libraries, and Chromium datasets.
2. _Everything is a view_ - it's often useful to see data that traverses many relationships at once. It is also useful to filter an item on features of its ancestor - for example, you might want to see all Chromium libraries whose ancestor specimen had `species == homo_sapiens`. To both these ends, the database schema implements a system of views wherein these relationships are traversed and useful fields are "brought up" so-to-speak, so that a client can filter against these ancestors. It is then up to the query to shape this data.
3. _Don't repeat column names_ - Because PostgresSQL [creates a composite type for every table and view](https://www.postgresql.org/docs/current/rowtypes.html), it is possible to select an entire row as one **record**. As long as you have a way for the application to read that record into a single type, then you can do something like this:
   ```sql
   select suspension_detailed from suspension_detailed;
   ```

## Application

The application is split into [cellnoor-types](./crates/cellnoor-types) and [cellnoor](./crates/cellnoor). The former contains the data models, while the latter contains the actual application.

### Reading and Writing Data

Where possible, we create "record" types that model a single "record" from a given table or view. For example, in [cellnoor-types/institution.rs](./crates/cellnoor-types/src/institution.rs):

```rust
use crate::{Id, NoId};

mod record {
    pub struct InstitutionRecord<T> {
        pub id: T,
        pub name: NonemptyString,
        pub microsoft_entra_tenant_id: Uuid,
    }
}

pub type NewInstitution = InstitutionRecord<NoId>;

pub type SavedInstitutionRecord = InstitutionRecord<Id>;
```

Because `tokio-postgres` has no "flatten" attribute, and we don't want to rewrite fieldnames more than necessary, the "record" struct has a parameterized `id` field for many of the models. In this case, that allows us to read an institution directly from the database using `tokio-postgres` (simplified code below):

```rust
async fn select_institution(client: tokio_postgres::Client) {
    let institution: SavedInstitutionRecord = x
        .query_one_scalar("select institution from institution", &[])
        .await
        .unwrap();
}
```

At the same time, it means the API layer does not accept an `id` field, since `NewInstitution` has `id: NoId`, which is a zero-sized struct.

### Querying Data

The most complex piece of the codebase is the query machinery. For each table or view, we define a Rust enum of fields on which we can filter. To maintain type-safety, we encode the allowed filters in this enum. Take a basic field, like a UUID, for example. Here is a (non-exhaustive) enum of operators that can be used to filter any data type:

```rust
/// A comparison operator for any scalar value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "{T}Operator"))]
pub enum Operator<T> {
    /// equals (=)
    Eq(T),
    /// less than (<)
    Lt(T),
    /// less than or equal to (<=)
    Lte(T),
    /// greater than (>)
    Gt(T),
    /// greater than or equal to (>=)
    Gte(T),
    /// is contained in (= any($1))
    In(Vec<T>),
    /// equals (=), but (de)serializes as '{"field": "value"}' instead of
    /// '{"field": {"eq": "value"}}'
    #[cfg(feature = "serde")]
    #[serde(untagged)]
    ImplicitEq(T),
}

// Define a type-alias to make it easier to work with
pub type UuidOperator = Operator<Uuid>;
```

Note that this has a convenient JSON serialization for consumers. Now, going back to our institution example, we define an enum of an institution's fields as well as the allowed data types:

```rust
// These two macros just wrap a bunch of derives at once
use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, UuidOperator}
};

#[predicate_enum]
#[strum(prefix = "(institution).")]
#[strum_discriminants(name(InstitutionField), sort_field_enum)]
pub enum InstitutionPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    MicrosoftEntraTenantId(UuidOperator),
}
```

The important bit here is that the `#[predicate_enum]` macro derives [`strum::EnumDiscriminants`](https://docs.rs/strum/latest/strum/derive.EnumDiscriminants.html), which we name `InstitutionField`. Now, we get an enum of the institution's fields for free, which we can use for sorting results in the application. We also use this field-enum to construct insert-statements. For a `NewInstitution`, we implement a trait called [`Insert`](./crates/cellnoor/src/db.rs), which tells our database machinery how to convert a Rust struct to a list of fieldname-value pairs (simplified code shown):

```rust
impl Insert for NewInstitution {
    type Field = InstitutionField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use InstitutionField::*;
        let Self {
            name,
            microsoft_entra_tenant_id,
        } = self;

        vec![
            (Name, name),
            (MicrosoftEntraTenantId, microsoft_entra_tenant_id),
        ]
    }
}
```

This documentation is incomplete, but it captures the most complex aspects of the interaction between database and application.
