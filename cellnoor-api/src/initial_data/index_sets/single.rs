use anyhow::ensure;
use cellnoor_schema::{single_index_sets, sql_types::CaseInsensitiveText};
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::initial_data::{
    Upsert,
    index_sets::common::{
        DNA_REGEX, INDEX_SET_NAME_ERROR_MESSAGE, INDEX_SET_NAME_REGEX, IndexSetName,
        insert_kit_name,
    },
};

#[derive(Clone, serde::Deserialize)]
pub(super) struct SingleIndexSet(String, [StringWrapper; 4]);

impl SingleIndexSet {
    fn validate(&self) -> anyhow::Result<()> {
        let Self(index_set_name, sequences) = self;

        ensure!(
            INDEX_SET_NAME_REGEX.is_match(index_set_name),
            INDEX_SET_NAME_ERROR_MESSAGE
        );

        ensure!(
            sequences
                .iter()
                .all(|StringWrapper(s)| DNA_REGEX.is_match(s)),
            "invalid DNA sequences: {sequences:?}"
        );

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = single_index_sets, check_for_backend(diesel::pg::Pg))]
struct SingleIndexSetInsertion<'a> {
    name: &'a str,
    kit: &'a str,
    well: &'a str,
    sequences: Vec<Option<&'a StringWrapper>>,
}

impl Upsert for Vec<SingleIndexSet> {
    async fn upsert(self, mut db_conn: &AsyncPgConnection) -> anyhow::Result<()> {
        self.iter().try_for_each(SingleIndexSet::validate)?;

        #[allow(clippy::get_first)]
        let Some(SingleIndexSet(index_set_name, ..)) = self.get(0).cloned() else {
            return Ok(());
        };

        let kit_name = index_set_name.kit_name()?;
        insert_kit_name(kit_name, db_conn).await?;

        let mut insertables = Vec::with_capacity(self.len());
        for SingleIndexSet(index_set_name, sequences) in &self {
            let well_name = index_set_name.well_name()?;

            insertables.push(SingleIndexSetInsertion {
                name: index_set_name,
                kit: kit_name,
                well: well_name,
                sequences: sequences.iter().map(Some).collect(),
            });
        }

        diesel::insert_into(single_index_sets::table)
            .values(insertables)
            .on_conflict_do_nothing()
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct StringWrapper(String);

impl ToSql<CaseInsensitiveText, Pg> for StringWrapper {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <String as ToSql<Text, Pg>>::to_sql(&self.0, out)
    }
}
