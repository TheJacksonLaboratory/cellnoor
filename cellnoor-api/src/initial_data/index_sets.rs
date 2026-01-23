use std::collections::HashMap;

pub(crate) use common::IndexSetName;
use diesel_async::AsyncPgConnection;
use serde::de::DeserializeOwned;
use tokio::task::JoinSet;
use tracing_subscriber::filter::FilterExt;
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
    db_conn: &AsyncPgConnection,
) -> anyhow::Result<()> {
    download_and_insert_index_sets::<HashMap<String, DualIndexSet>>(file_urls, http_client, db_conn)
        .await
}

pub(super) async fn download_and_insert_single_index_sets(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: &AsyncPgConnection,
) -> anyhow::Result<()> {
    download_and_insert_index_sets::<Vec<SingleIndexSet>>(file_urls, http_client, db_conn).await
}

async fn download_and_insert_index_sets<T>(
    file_urls: Vec<Url>,
    http_client: reqwest::Client,
    db_conn: &AsyncPgConnection,
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
    let index_sets = index_sets.into_iter().map(|s| s.upsert(db_conn));
    futures::future::try_join_all(index_sets).await?;

    Ok(())
}

async fn download_json<T: DeserializeOwned>(
    http_client: reqwest::Client,
    url: Url,
) -> anyhow::Result<T> {
    Ok(http_client.get(url).send().await?.json().await?)
}
