use std::collections::HashMap;

pub(crate) use common::IndexSetName;
use serde::de::DeserializeOwned;
use tokio::task::JoinSet;
use url::Url;

use crate::initial_data::{
    Upsert,
    index_sets::{dual::DualIndexSet, single::SingleIndexSet},
};

mod common;
mod dual;
mod single;

pub(super) async fn download_and_insert_dual_index_sets(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: &deadpool_diesel::postgres::Connection,
) -> anyhow::Result<()> {
    download_and_insert_index_sets::<HashMap<String, DualIndexSet>>(file_urls, http_client, db_conn)
        .await
}

pub(super) async fn download_and_insert_single_index_sets(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: &deadpool_diesel::postgres::Connection,
) -> anyhow::Result<()> {
    download_and_insert_index_sets::<Vec<SingleIndexSet>>(file_urls, http_client, db_conn).await
}

async fn download_and_insert_index_sets<T>(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: &deadpool_diesel::postgres::Connection,
) -> anyhow::Result<()>
where
    T: 'static + DeserializeOwned + Send + Upsert,
{
    // let downloads = JoinSet::new();
    // for url in file_urls {
    //     downloads.spawn(download_json::<T>(http_client.clone(), url));
    // }
    let downloads: JoinSet<_> = file_urls
        .into_iter()
        .map(|url| download_json::<T>(http_client.clone(), url))
        .collect();

    let index_sets = downloads
        .join_all()
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    // A for-loop is fine because this is like 10 URLs max, and each of these is a
    // bulk insert
    for sets in index_sets {
        db_conn
            .interact(|db_conn| sets.upsert(db_conn))
            .await
            .unwrap()?;
    }

    Ok(())
}

async fn download_json<T: DeserializeOwned>(
    http_client: reqwest::Client,
    url: Url,
) -> anyhow::Result<T> {
    Ok(http_client.get(url).send().await?.json().await?)
}
