use anyhow::Context;
use cellnoor_api::{api, settings::Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::read().context("failed to read app configuration")?;

    api::serve(&settings).await?;

    Ok(())
}
