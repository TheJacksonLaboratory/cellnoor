pub mod settings;
pub mod state;

#[cfg(test)]
mod tests {
    #[tokio::test(flavor = "multi_thread")]
    async fn test() {}
}
