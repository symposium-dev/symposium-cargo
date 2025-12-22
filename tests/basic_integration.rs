use anyhow::Result;
use expect_test::expect;
use sacp::DynComponent;
use sacp_conductor::Conductor;
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
        Conductor::new(
            "test-conductor".to_string(),
            vec![
                DynComponent::new(proxy),
                DynComponent::new(elizacp::ElizaAgent::new()),
            ],
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
async fn test_cargo_check_with_elizacp() -> Result<()> {
    let response = run_cargo_tool("cargo_check").await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"command\":\"cargo check  --message-format json\",\"exit_code\":101,\"messages\":[{\"package_id\":\"path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0\",\"reason\":\"compiler-message\",\"rendered_message\":\"error[E0425]: cannot find value `error` in this scope\\n --> src/main.rs:2:5\\n  |\\n2 |     error\\n  |     ^^^^^ not found in this scope\\n\\n\"},{\"package_id\":\"path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0\",\"reason\":\"compiler-message\",\"rendered_message\":\"For more information about this error, try `rustc --explain E0425`.\\n\"},{\"reason\":\"build-finished\",\"success\":false}],\"stderr\":\"    Checking test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\\nerror: could not compile `test-project` (bin \\\"test-project\\\") due to 1 previous error\",\"success\":false}", meta: None }), annotations: None }], structured_content: Some(Object {"command": String("cargo check  --message-format json"), "exit_code": Number(101), "messages": Array [Object {"package_id": String("path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0"), "reason": String("compiler-message"), "rendered_message": String("error[E0425]: cannot find value `error` in this scope\n --> src/main.rs:2:5\n  |\n2 |     error\n  |     ^^^^^ not found in this scope\n\n")}, Object {"package_id": String("path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0"), "reason": String("compiler-message"), "rendered_message": String("For more information about this error, try `rustc --explain E0425`.\n")}, Object {"reason": String("build-finished"), "success": Bool(false)}], "stderr": String("    Checking test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\nerror: could not compile `test-project` (bin \"test-project\") due to 1 previous error"), "success": Bool(false)}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}

#[tokio::test]
async fn test_cargo_build_with_elizacp() -> Result<()> {
    let response = run_cargo_tool("cargo_build").await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"command\":\"cargo build  --message-format json\",\"exit_code\":101,\"messages\":[{\"package_id\":\"path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0\",\"reason\":\"compiler-message\",\"rendered_message\":\"error[E0425]: cannot find value `error` in this scope\\n --> src/main.rs:2:5\\n  |\\n2 |     error\\n  |     ^^^^^ not found in this scope\\n\\n\"},{\"package_id\":\"path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0\",\"reason\":\"compiler-message\",\"rendered_message\":\"For more information about this error, try `rustc --explain E0425`.\\n\"},{\"reason\":\"build-finished\",\"success\":false}],\"stderr\":\"   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\\nerror: could not compile `test-project` (bin \\\"test-project\\\") due to 1 previous error\",\"success\":false}", meta: None }), annotations: None }], structured_content: Some(Object {"command": String("cargo build  --message-format json"), "exit_code": Number(101), "messages": Array [Object {"package_id": String("path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0"), "reason": String("compiler-message"), "rendered_message": String("error[E0425]: cannot find value `error` in this scope\n --> src/main.rs:2:5\n  |\n2 |     error\n  |     ^^^^^ not found in this scope\n\n")}, Object {"package_id": String("path+file:///home/gh-jackh726/symposium/symposium-cargo/tests/test-project#0.1.0"), "reason": String("compiler-message"), "rendered_message": String("For more information about this error, try `rustc --explain E0425`.\n")}, Object {"reason": String("build-finished"), "success": Bool(false)}], "stderr": String("   Compiling test-project v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo/tests/test-project)\nerror: could not compile `test-project` (bin \"test-project\") due to 1 previous error"), "success": Bool(false)}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}
