use std::collections::HashMap;

use tokio::task::JoinSet;
use url::Url;

use crate::initial_data::{
    Upsert,
    index_sets::{dual::DualIndexSet, single::SingleIndexSet},
};

mod common;
mod dual;
mod single;

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum IndexSets {
    Single(Vec<SingleIndexSet>),
    Dual(HashMap<String, DualIndexSet>),
}

async fn download_index_sets(http_client: reqwest::Client, url: Url) -> anyhow::Result<IndexSets> {
    Ok(http_client.get(url).send().await?.json().await?)
}

pub(super) async fn download_and_insert_index_sets(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: deadpool_diesel::postgres::Connection,
) -> anyhow::Result<()> {
    let downloads: JoinSet<_> = file_urls
        .into_iter()
        .map(|url| download_index_sets(http_client.clone(), url))
        .collect();

    let index_sets: Vec<_> = downloads
        .join_all()
        .await
        .into_iter()
        .collect::<anyhow::Result<_>>()?;

    // A for-loop is fine because this is like 10 URLs max, and each of these is a
    // bulk insert
    for sets in index_sets {
        match sets {
            IndexSets::Dual(s) => db_conn
                .interact(|db_conn| s.upsert(db_conn))
                .await
                .unwrap()?,
            IndexSets::Single(s) => db_conn
                .interact(|db_conn| s.upsert(db_conn))
                .await
                .unwrap()?,
        }
    }

    Ok(())
}
