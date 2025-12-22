mod cargo_proxy;
mod cargo_command;

use anyhow::Result;
use sacp::component::Component;
use sacp::ProxyToConductor;

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

pub struct CargoProxy;

impl Component for CargoProxy {
    async fn serve(self, client: impl Component) -> Result<(), sacp::Error> {
        ProxyToConductor::builder()
            .name("cargo-proxy")
            .with_mcp_server(cargo_proxy::build_server())
            .serve(client)
            .await
    }
}
