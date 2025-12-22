use anyhow::Result;
use expect_test::expect;
use sacp::DynComponent;
use sacp_conductor::Conductor;
use symposium_cargo::CargoProxy;

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

#[tokio::test]
async fn test_cargo_check_with_elizacp() -> Result<()> {
    setup_tracing();

    // Create the component chain: CargoProxy -> ElizACP
    let proxy = CargoProxy;

    // Send a tool invocation to cargo_proxy
    // ElizACP expects format: "Use tool <server>::<tool> with <json_params>"
    let response = yopo::prompt(
        Conductor::new(
            "test-conductor".to_string(),
            vec![
                DynComponent::new(proxy),
                DynComponent::new(elizacp::ElizaAgent::new()),
            ],
            Default::default(),
        ),
        r#"Use tool cargo-mcp::cargo_check with {}"#,
    )
    .await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"command\":\"cargo check  --message-format json\",\"exit_code\":0,\"messages\":[{\"reason\":\"build-finished\",\"success\":true}],\"stderr\":\"    Checking symposium-cargo v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo)\\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s\",\"success\":true}", meta: None }), annotations: None }], structured_content: Some(Object {"command": String("cargo check  --message-format json"), "exit_code": Number(0), "messages": Array [Object {"reason": String("build-finished"), "success": Bool(true)}], "stderr": String("    Checking symposium-cargo v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo)\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s"), "success": Bool(true)}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}

#[tokio::test]
async fn test_cargo_build_with_elizacp() -> Result<()> {
    setup_tracing();

    // Create the component chain: CargoProxy -> ElizACP
    let proxy = CargoProxy;

    // Send a tool invocation to cargo_proxy
    // ElizACP expects format: "Use tool <server>::<tool> with <json_params>"
    let response = yopo::prompt(
        Conductor::new(
            "test-conductor".to_string(),
            vec![
                DynComponent::new(proxy),
                DynComponent::new(elizacp::ElizaAgent::new()),
            ],
            Default::default(),
        ),
        r#"Use tool cargo-mcp::cargo_build with {}"#,
    )
    .await?;

    expect![[r#"OK: CallToolResult { content: [Annotated { raw: Text(RawTextContent { text: "{\"command\":\"cargo build  --message-format json\",\"exit_code\":0,\"messages\":[{\"reason\":\"build-finished\",\"success\":true}],\"stderr\":\"   Compiling symposium-cargo v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo)\\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.65s\",\"success\":true}", meta: None }), annotations: None }], structured_content: Some(Object {"command": String("cargo build  --message-format json"), "exit_code": Number(0), "messages": Array [Object {"reason": String("build-finished"), "success": Bool(true)}], "stderr": String("   Compiling symposium-cargo v0.1.0 (/home/gh-jackh726/symposium/symposium-cargo)\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.65s"), "success": Bool(true)}), is_error: Some(false), meta: None }"#]].assert_eq(&response);

    Ok(())
}
