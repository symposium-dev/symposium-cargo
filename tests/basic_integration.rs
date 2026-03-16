use anyhow::Result;
use sacp_conductor::{Conductor, ProxiesAndAgent};
use std::path::PathBuf;
use symposium_cargo::CargoProxy;

fn get_test_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test-project")
}

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .compact()
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .try_init();
}

async fn run_cargo_tool(tool: &str) -> Result<String> {
    setup_tracing();
    let proxy = CargoProxy::default();
    let test_project = get_test_project_path();

    Ok(yopo::prompt(
        Conductor::new_agent(
            "test-conductor".to_string(),
            ProxiesAndAgent::new(elizacp::ElizaAgent::new()).proxy(proxy),
            Default::default(),
        ),
        &format!(
            r#"Use tool cargo-mcp::{} with {{"cwd": "{}"}}"#,
            tool,
            test_project.display()
        ),
    )
    .await?)
}

#[tokio::test]
async fn test_cargo_check() -> Result<()> {
    let response = run_cargo_tool("cargo_check").await?;
    assert!(response.contains("cannot find value `error` in this scope"));
    Ok(())
}

#[tokio::test]
async fn test_cargo_build() -> Result<()> {
    let response = run_cargo_tool("cargo_build").await?;
    assert!(response.contains("cannot find value `error` in this scope"));
    Ok(())
}
#[tokio::test]
async fn test_cargo_test() -> Result<()> {
    let response = run_cargo_tool("cargo_test").await?;
    assert!(response.contains("cannot find value `error` in this scope"));
    Ok(())
}
