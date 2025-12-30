mod cargo_command;
pub mod cargo_mcp;

use anyhow::Result;
pub use cargo_mcp::build_mcp_server;
use sacp::ProxyToConductor;
use sacp::component::Component;

pub struct CargoProxy;

impl Component for CargoProxy {
    async fn serve(self, client: impl Component) -> Result<(), sacp::Error> {
        ProxyToConductor::builder()
            .name("cargo-proxy")
            .with_mcp_server(build_mcp_server())
            .serve(client)
            .await
    }
}
