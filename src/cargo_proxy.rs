use sacp::{
    ProxyToConductor, mcp_server::McpServer,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::cargo_command::{CargoCommandParams, execute_cargo_command_async};

#[derive(Serialize, Deserialize, JsonSchema)]
struct EmptyParams {}

pub fn build_server() -> McpServer<ProxyToConductor, impl sacp::JrResponder<ProxyToConductor>> {
    McpServer::builder("cargo-mcp".to_string())
        .instructions(indoc::indoc! {"
            Run cargo commands.
        "})
        .tool_fn_mut(
            "cargo_check",
            indoc::indoc! {r#"
                Runs cargo check.
            "#},
            async move |_input: EmptyParams, _mcp_cx: sacp::mcp_server::McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command_async(CargoCommandParams { command: "check".to_string(), args: vec![], cwd: None }).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_build",
            indoc::indoc! {r#"
                Runs cargo build.
            "#},
            async move |_input: EmptyParams, _mcp_cx: sacp::mcp_server::McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command_async(CargoCommandParams { command: "build".to_string(), args: vec![], cwd: None }).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .build()
}
