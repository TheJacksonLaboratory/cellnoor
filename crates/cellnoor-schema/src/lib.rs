mod schema;

use diesel::{deserialize::FromSql, pg::Pg, sql_types::Text};
pub use schema::*;

use crate::schema::sql_types::CaseInsensitiveText;

impl FromSql<CaseInsensitiveText, Pg> for String {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        <Self as FromSql<Text, Pg>>::from_sql(bytes)
    }
}
