use anyhow::Result;
use sacp::component::Component;
use symposium_cargo::CargoProxy;

/// Run the proxy as a standalone binary connected to stdio
pub async fn run() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting cargo-proxy");

    CargoProxy.serve(sacp_tokio::Stdio::new()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
