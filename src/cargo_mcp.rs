use crate::cargo_command::execute_cargo_command;
use sacp::{
    ProxyToConductor,
    mcp_server::{McpContext, McpServer},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoCommandInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoTestInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_arg: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoAddInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoCleanInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoRemoveInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoRunInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CargoUpdateInputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

pub fn build_mcp_server() -> McpServer<ProxyToConductor, impl sacp::JrResponder<ProxyToConductor>> {
    McpServer::builder("cargo-mcp".to_string())
        .instructions(indoc::indoc! {"
            Run cargo commands. When possible, always use this instead of calling a shell command.
        "})
        .tool_fn_mut(
            "cargo_check",
            indoc::indoc! {r#"
                Runs cargo check.
            "#},
            async move |input: CargoCommandInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command("check", vec![], input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_build",
            indoc::indoc! {r#"
                Runs cargo build.
            "#},
            async move |input: CargoCommandInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                Ok(execute_cargo_command("build", vec![], input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_test",
            indoc::indoc! {r#"
                Runs cargo test. Optionally specify a test name or pattern to run specific tests.
            "#},
            async move |input: CargoTestInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let args = if let Some(test_arg) = input.test_arg.as_deref() {
                    vec![test_arg]
                } else {
                    vec![]
                };
                Ok(execute_cargo_command("test", args, input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_add",
            indoc::indoc! {r#"
                Runs `cargo add <package> [extra args]`.
            "#},
            async move |input: CargoAddInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let mut args: Vec<&str> = Vec::new();
                args.push(&input.package);
                if let Some(extra) = &input.extra_args {
                    args.extend(extra.iter().map(|s| s.as_str()));
                }

                Ok(execute_cargo_command("add", args, input.cwd, false).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_clean",
            indoc::indoc! {r#"
                Runs `cargo clean [extra args]`.
            "#},
            async move |input: CargoCleanInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let mut args: Vec<&str> = Vec::new();
                if let Some(extra) = &input.extra_args {
                    args.extend(extra.iter().map(|s| s.as_str()));
                }

                Ok(execute_cargo_command("clean", args, input.cwd, true).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_remove",
            indoc::indoc! {r#"
                Runs `cargo remove <package> [extra args]`.
            "#},
            async move |input: CargoRemoveInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let mut args: Vec<&str> = Vec::new();
                args.push(&input.package);
                if let Some(extra) = &input.extra_args {
                    args.extend(extra.iter().map(|s| s.as_str()));
                }

                Ok(execute_cargo_command("remove", args, input.cwd, true).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_run",
            indoc::indoc! {r#"
                Runs `cargo run [extra args]`.
            "#},
            async move |input: CargoRunInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let mut args: Vec<&str> = Vec::new();
                let release = input.release.unwrap_or(false).to_string();
                args.push(&release);
                if let Some(a) = &input.args {
                    args.extend(a.iter().map(|s| s.as_str()));
                }

                Ok(execute_cargo_command("run", args, input.cwd, true).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .tool_fn_mut(
            "cargo_update",
            indoc::indoc! {r#"
                Runs `cargo update`. Optionally specify `package` (uses `-p`) and extra args.
            "#},
            async move |input: CargoUpdateInputs, _mcp_cx: McpContext<ProxyToConductor>| {
                let mut args: Vec<&str> = Vec::new();
                if let Some(pkg) = input.package.as_deref() {
                    args.push("-p");
                    args.push(pkg);
                }
                if let Some(extra) = &input.extra_args {
                    args.extend(extra.iter().map(|s| s.as_str()));
                }

                Ok(execute_cargo_command("update", args, input.cwd, true).await?)
            },
            sacp::tool_fn_mut!(),
        )
        .build()
}
