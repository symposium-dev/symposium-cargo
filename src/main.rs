use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    symposium_cargo::run().await
}
