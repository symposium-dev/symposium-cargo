mod cargo_command;
pub mod cargo_mcp;

use anyhow::Result;
pub use cargo_mcp::build_mcp_server;
use sacp::ProxyToConductor as ProxyToConductorBuilder;
use sacp::component::Component;
use sacp::link::{ConductorToProxy, ProxyToConductor};

pub struct CargoProxy;

impl Component<ProxyToConductor> for CargoProxy {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        ProxyToConductorBuilder::builder()
            .name("cargo-proxy")
            .with_mcp_server(build_mcp_server())
            .serve(client)
            .await
    }
}
