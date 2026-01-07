use anyhow::Result;
use expect_test::expect;
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
    let proxy = CargoProxy;
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

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"build_success\":false,\"command\":\"cargo check  --message-format json\",\"exit_code\":101,\"messages\":[\"error[E0425]: cannot find value `error` in this scope\\n --> src/main.rs:2:5\\n  |\\n2 |     error\\n  |     ^^^^^ not found in this scope\\n\\n\",\"For more information about this error, try `rustc --explain E0425`.\\n\"],\"stderr\":\"    Checking test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\\nerror: could not compile `test-project` (bin \\\"test-project\\\") due to 1 previous error\"}", meta: None }), annotations: None }], structured_content: Some(Object {"build_success": Bool(false), "command": String("cargo check  --message-format json"), "exit_code": Number(101), "messages": Array [String("error[E0425]: cannot find value `error` in this scope\n --> src/main.rs:2:5\n  |\n2 |     error\n  |     ^^^^^ not found in this scope\n\n"), String("For more information about this error, try `rustc --explain E0425`.\n")], "stderr": String("    Checking test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\nerror: could not compile `test-project` (bin \"test-project\") due to 1 previous error")}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}

#[tokio::test]
async fn test_cargo_build() -> Result<()> {
    let response = run_cargo_tool("cargo_build").await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"build_success\":false,\"command\":\"cargo build  --message-format json\",\"exit_code\":101,\"messages\":[\"error[E0425]: cannot find value `error` in this scope\\n --> src/main.rs:2:5\\n  |\\n2 |     error\\n  |     ^^^^^ not found in this scope\\n\\n\",\"For more information about this error, try `rustc --explain E0425`.\\n\"],\"stderr\":\"   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\\nerror: could not compile `test-project` (bin \\\"test-project\\\") due to 1 previous error\"}", meta: None }), annotations: None }], structured_content: Some(Object {"build_success": Bool(false), "command": String("cargo build  --message-format json"), "exit_code": Number(101), "messages": Array [String("error[E0425]: cannot find value `error` in this scope\n --> src/main.rs:2:5\n  |\n2 |     error\n  |     ^^^^^ not found in this scope\n\n"), String("For more information about this error, try `rustc --explain E0425`.\n")], "stderr": String("   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\nerror: could not compile `test-project` (bin \"test-project\") due to 1 previous error")}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}
#[tokio::test]
async fn test_cargo_test() -> Result<()> {
    let response = run_cargo_tool("cargo_test").await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"build_success\":false,\"command\":\"cargo test  --message-format json\",\"exit_code\":101,\"messages\":[\"error[E0425]: cannot find value `error` in this scope\\n --> src/main.rs:2:5\\n  |\\n2 |     error\\n  |     ^^^^^ not found in this scope\\n\\n\",\"For more information about this error, try `rustc --explain E0425`.\\n\"],\"stderr\":\"   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\\nerror: could not compile `test-project` (bin \\\"test-project\\\" test) due to 1 previous error\"}", meta: None }), annotations: None }], structured_content: Some(Object {"build_success": Bool(false), "command": String("cargo test  --message-format json"), "exit_code": Number(101), "messages": Array [String("error[E0425]: cannot find value `error` in this scope\n --> src/main.rs:2:5\n  |\n2 |     error\n  |     ^^^^^ not found in this scope\n\n"), String("For more information about this error, try `rustc --explain E0425`.\n")], "stderr": String("   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\nerror: could not compile `test-project` (bin \"test-project\" test) due to 1 previous error")}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}
