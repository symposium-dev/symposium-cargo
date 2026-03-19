use anyhow::Result;
use sacp::schema::{
    ContentBlock, InitializeRequest, ProtocolVersion, SessionNotification, TextContent,
};
use sacp::util::MatchDispatch;
use sacp::{Client, ConnectionTo};
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;

use sacp_conductor::{ConductorImpl, ProxiesAndAgent};
use symposium_cargo::CargoProxy;
use symposium_cargo::cargo_command::execute_cargo_command;

static STAGE0_CONTENT: &str = "stage 0: pass -> fail";
static STAGE1_CONTENT: &str = "stage 1: fail -> pass";

async fn make_fail(project_root: &PathBuf) -> Result<()> {
    let _ = tokio::fs::create_dir_all(project_root.join("src")).await;
    if let Ok(mut f) = tokio::fs::File::create(project_root.join("src/lib.rs")).await {
        f.write(
            "#[cfg(test)] mod tests { #[test] fn it_fails() { assert_eq!(2 + 2, 5); } }".as_bytes(),
        )
        .await?;
    }
    Ok(())
}

async fn make_pass(project_root: &PathBuf) -> Result<()> {
    let _ = tokio::fs::create_dir_all(project_root.join("src")).await;
    if let Ok(mut f) = tokio::fs::File::create(project_root.join("src/lib.rs")).await {
        f.write(
            "#[cfg(test)] mod tests { #[test] fn it_works() { assert_eq!(2 + 2, 4); } }".as_bytes(),
        )
        .await?;
    }
    Ok(())
}

mod agent {
    use anyhow::Result;
    use sacp::schema::{
        AgentCapabilities, ContentBlock, ContentChunk, InitializeRequest, InitializeResponse,
        LoadSessionRequest, LoadSessionResponse, NewSessionRequest, NewSessionResponse,
        PromptRequest, PromptResponse, SessionNotification, SessionUpdate, StopReason, TextContent,
    };
    use sacp::{Agent, Client, ConnectTo, ConnectionTo};

    use serde_json;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{STAGE0_CONTENT, STAGE1_CONTENT};

    #[derive(Clone)]
    pub struct TestAgent {
        project_root: PathBuf,
        sessions: Arc<Mutex<HashMap<String, ConnectionTo<Client>>>>,
        stage: u8,
    }

    impl TestAgent {
        pub fn new(project_root: PathBuf) -> Self {
            let sessions = Arc::new(Mutex::new(HashMap::new()));

            TestAgent {
                project_root,
                sessions,
                stage: 0,
            }
        }
    }

    impl ConnectTo<Client> for TestAgent {
        async fn connect_to(
            mut self,
            client: impl ConnectTo<Agent>,
        ) -> std::result::Result<(), sacp::Error> {
            let sessions = self.sessions.clone();

            Agent
                .builder()
                .name("test-agent")
                .on_receive_request(
                    async move |initialize: InitializeRequest, responder, _cx| {
                        responder.respond(
                            InitializeResponse::new(initialize.protocol_version)
                                .agent_capabilities(AgentCapabilities::new()),
                        )
                    },
                    sacp::on_receive_request!(),
                )
                .on_receive_request(
                    {
                        let sessions = sessions.clone();
                        async move |_request: NewSessionRequest, responder, _cx| {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            let session_id = format!("test-{}", now);
                            sessions
                                .lock()
                                .unwrap()
                                .insert(session_id.clone().to_string(), _cx.clone());
                            responder.respond(NewSessionResponse::new(session_id))
                        }
                    },
                    sacp::on_receive_request!(),
                )
                .on_receive_request(
                    {
                        let sessions = sessions.clone();
                        async move |request: LoadSessionRequest, responder, _cx| {
                            sessions
                                .lock()
                                .unwrap()
                                .insert(request.session_id.clone().to_string(), _cx.clone());
                            responder.respond(LoadSessionResponse::new())
                        }
                    },
                    sacp::on_receive_request!(),
                )
                .on_receive_request(
                    {
                        let sessions = sessions.clone();
                        async move |request: PromptRequest,
                                    responder,
                                    cx|
                                    -> Result<(), sacp::Error> {
                            sessions
                                .lock()
                                .unwrap()
                                .insert(request.session_id.clone().to_string(), cx.clone());

                            let req_json = serde_json::to_string(&request).unwrap_or_default();
                            println!("TestAgent received PromptRequest JSON: {}", req_json);
                            match self.stage {
                                0 => {
                                    assert!(req_json.contains("start"));
                                    super::make_fail(&self.project_root).await?;

                                    cx.send_notification(SessionNotification::new(
                                        request.session_id.clone(),
                                        SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                            ContentBlock::Text(TextContent::new(
                                                STAGE0_CONTENT.to_string(),
                                            )),
                                        )),
                                    ))?;
                                }
                                1 => {
                                    assert!(
                                        req_json
                                            .contains("current project doesn't compile/pass tests")
                                    );
                                    super::make_pass(&self.project_root).await?;

                                    cx.send_notification(SessionNotification::new(
                                        request.session_id.clone(),
                                        SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                            ContentBlock::Text(TextContent::new(
                                                STAGE1_CONTENT.to_string(),
                                            )),
                                        )),
                                    ))?;
                                }
                                _ => {
                                    panic!("unexpected stage");
                                }
                            }
                            self.stage += 1;

                            responder.respond(PromptResponse::new(StopReason::EndTurn))
                        }
                    },
                    sacp::on_receive_request!(),
                )
                .connect_to(client)
                .await
        }
    }
}

async fn write_cargo_toml(path: &PathBuf) -> Result<()> {
    let mut f = tokio::fs::File::create(path.join("Cargo.toml")).await?;
    f.write(
        r#"[package]
name = "temp_project"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
"#
        .as_bytes(),
    )
    .await?;
    Ok(())
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

#[tokio::test]
async fn subagent_flow_simulation() -> Result<()> {
    setup_tracing();

    // 1) Create temporary cargo project with passing tests
    let dir = tempdir()?;
    let project_path = dir.path().to_path_buf();
    write_cargo_toml(&project_path).await?;
    make_pass(&project_path).await?;

    // Verify tests pass initially
    let res_pass = execute_cargo_command(
        "test",
        vec![],
        Some(project_path.to_string_lossy().into()),
        false,
    )
    .await?;
    assert_eq!(res_pass.exit_code, Some(0), "initial tests should pass");

    // 2) Start a Conductor with our `TestAgent` and the `CargoProxy` so we can send a single
    // prompt which the agent + proxy will orchestrate. The `TestAgent` exposes a handle we can
    // use to apply filesystem fixes when prompted.
    let agent = agent::TestAgent::new(project_path.clone());

    let proxy = CargoProxy {
        workspace_path: Some(project_path.to_string_lossy().to_string()),
    };

    let component = ConductorImpl::new_agent(
        "test-conductor".to_string(),
        ProxiesAndAgent::new(agent).proxy(proxy),
        Default::default(),
    );
    Client.builder()
        .connect_with(component, |connection: ConnectionTo<sacp::Agent>| async move {
            // Initialize the agent
            let _init_response = connection
                .send_request(InitializeRequest::new(ProtocolVersion::LATEST))
                .block_task()
                .await?;

            let mut session = connection
                .build_session(PathBuf::from("."))
                .block_task()
                .start_session()
                .await?;

            session.send_prompt("start".to_string())?;

            let mut stage = 0;
            let mut do_break = false;
            loop {
                if do_break {
                    break;
                }
                let update = session.read_update().await?;
                match update {
                    sacp::SessionMessage::SessionMessage(message) => {
                        MatchDispatch::new(message)
                            .if_notification(async |notification: SessionNotification| {
                                match notification.update {
                                    sacp::schema::SessionUpdate::AgentMessageChunk(
                                        content_chunk,
                                    ) => {
                                        match stage {
                                            0 => {
                                                assert_eq!(&content_chunk.content, &ContentBlock::Text(TextContent::new(STAGE0_CONTENT.to_string())));
                                            }
                                            _ => {
                                                panic!("unexpected stage {}", stage);
                                            }
                                        }
                                    }
                                    sacp::schema::SessionUpdate::UserMessageChunk(content_chunk) => {
                                        match stage {
                                            1 => {
                                                assert_eq!(&content_chunk.content, &ContentBlock::Text(TextContent::new("Cargo tests passed after automated fix and `cargo fmt` was run.".to_string())));
                                                do_break = true;
                                            }
                                            _ => {
                                                panic!("unexpected stage {}", stage);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                stage += 1;
                                Ok(())
                            })
                            .await
                            .otherwise(async |_msg| Ok(()))
                            .await?;
                    }
                    sacp::SessionMessage::StopReason(stop_reason) => match stop_reason {
                        sacp::schema::StopReason::EndTurn => {}
                        sacp::schema::StopReason::MaxTokens => todo!(),
                        sacp::schema::StopReason::MaxTurnRequests => todo!(),
                        sacp::schema::StopReason::Refusal => todo!(),
                        sacp::schema::StopReason::Cancelled => todo!(),
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }

            Ok(())
        })
        .await?;

    // 3) Verify tests pass after the fix
    let res_fixed = execute_cargo_command(
        "test",
        vec![],
        Some(project_path.to_string_lossy().into()),
        false,
    )
    .await?;
    assert_eq!(
        res_fixed.exit_code,
        Some(0),
        "tests should pass after sub-agent fix"
    );

    Ok(())
}
