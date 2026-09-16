use anyhow::Context;
use cellnoor::{api, settings::Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap_or_default();
    let settings = Settings::read().context("failed to read app configuration")?;

    api::serve(&settings).await?;

    Ok(())
}
