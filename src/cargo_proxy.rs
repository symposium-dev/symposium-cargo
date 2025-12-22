use crate::cargo_command::execute_cargo_command;
use sacp::{ProxyToConductor, mcp_server::McpServer};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoCommandInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

pub fn build_server() -> McpServer<ProxyToConductor, impl sacp::JrResponder<ProxyToConductor>> {
    McpServer::builder("cargo-mcp".to_string())
        .instructions(indoc::indoc! {"
            Run cargo commands. When possible, always use this instead of calling a shell command.
        "})
        .tool_fn_mut(
            "cargo_check",
            indoc::indoc! {r#"
                Runs cargo check.
            "#},
            async move |input: CargoCommandInputs,
                        _mcp_cx: sacp::mcp_server::McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command("check", vec![], input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_build",
            indoc::indoc! {r#"
                Runs cargo build.
            "#},
            async move |input: CargoCommandInputs,
                        _mcp_cx: sacp::mcp_server::McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command("build", vec![], input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .build()
}
