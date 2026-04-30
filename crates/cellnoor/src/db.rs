pub fn create_pool(
    db_url: &str,
    max_size: Option<usize>,
) -> anyhow::Result<deadpool_postgres::Pool> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = db_url.expose_secret().to_owned().into();

    let mut builder = cfg.builder(deadpool_postgres::tokio_postgres::NoTls)?;
    if let Some(max_size) = max_size {
        builder = builder.max_size(max_size);
    }

    Ok(builder.build()?)
}
