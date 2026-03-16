pub mod cargo_command;
pub mod cargo_mcp;

use std::sync::Arc;

use anyhow::Result;
pub use cargo_mcp::build_mcp_server;
use sacp::component::Component;
use sacp::link::{ConductorToProxy, ProxyToConductor};
use sacp::schema::{ContentChunk, PromptRequest, SessionNotification, SessionUpdate, TextContent};
use sacp::{AgentPeer, ClientPeer, on_receive_request};
use tokio::sync::{RwLock, oneshot};

#[derive(Default)]
pub struct CargoProxy {
    pub workspace_path: Option<String>,
}

impl Component<ProxyToConductor> for CargoProxy {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        let cwd = Arc::new(RwLock::new(self.workspace_path));
        ProxyToConductor::builder()
            .name("cargo-proxy") 
            .with_mcp_server(build_mcp_server(cwd.clone()))
            .on_receive_request_from(
                ClientPeer,
                {
                    let cwd = cwd.clone();
                    async move |prompt_req: PromptRequest, req_cx, conn_cx| {
                        conn_cx
                            .send_request_to(AgentPeer, prompt_req.clone())
                            .on_receiving_ok_result(req_cx, {
                                let cwd = cwd.clone();
                                move |res, req_cx| async move {
                                    req_cx.respond(res.clone())?;
                                    match res.stop_reason {
                                        sacp::schema::StopReason::EndTurn => {
                                            let cwd_opt = cwd.read().await.clone();

                                            // Try running the test suite first. If you'd rather only run `check`, change to "check" here.
                                            let test_res = crate::cargo_command::execute_cargo_command("test", vec![], cwd_opt.clone(), false).await?;
                                            if let Some(0) = test_res.exit_code {
                                                // Tests passed — run `cargo fmt` (no JSON)
                                                let _fmt_res = crate::cargo_command::execute_cargo_command("fmt", vec![], cwd_opt, true).await?;

                                                let content = sacp::schema::ContentBlock::Text(TextContent::new("Cargo tests passed and `cargo fmt` was run.".to_string()));
                                                conn_cx.send_notification_to(ClientPeer, SessionNotification::new(prompt_req.session_id.clone(), SessionUpdate::UserMessageChunk(ContentChunk::new(content))))?;
                                                return Ok(());
                                            }

                                            let (sub_tx, sub_rx) = oneshot::channel();
                                            // Tests failed — prepare a short "fix" session for the agent to attempt minimal edits.
                                            conn_cx.build_session_cwd()?.on_session_start(async move |session| {
                                                let json = serde_json::to_string(&test_res)?;
                                                let failure_block = sacp::schema::ContentBlock::Text(TextContent::new(indoc::formatdoc! {
                                                    "The current project doesn't compile/pass tests. Here is the test/build output (JSON):\n\n{json}"
                                                }));

                                                let instructions = sacp::schema::ContentBlock::Text(TextContent::new(indoc::formatdoc! {
                                                    "Please attempt to make the project compile and pass tests. Keep changes minimal and local (one or two small edits). If the required change looks non-trivial, stop and report back so the user can intervene."
                                                }));

                                                // send a PromptRequest to the original session id with a one-line status.
                                                let mut blocks_to_send = vec![failure_block.clone(), instructions.clone()];
                                                let notify_instr = format!("When you finish reply with a short summary: whether you made a minimal fix that caused tests to pass, or that you stopped because required changes are non-trivial. If you made changes, include a one-line description of the change.");
                                                let notify_block = sacp::schema::ContentBlock::Text(TextContent::new(notify_instr));
                                                blocks_to_send.push(notify_block);

                                                let response = session.connection_cx().send_request_to(AgentPeer, PromptRequest::new(session.session_id().clone(), blocks_to_send));

                                                response.on_receiving_result(async move |res| {
                                                    sub_tx.send(res).map_err(|e| anyhow::anyhow!("Failed to send sub-session result: {:?}", e))?;

                                                    Ok(())
                                                })?;


                                                Ok(())
                                            })?;

                                            tokio::task::spawn(async move {
                                                let _ = sub_rx.await;

                                                // Re-run tests after the fix-session completed.
                                                let post_res = crate::cargo_command::execute_cargo_command("test", vec![], cwd_opt.clone(), false).await.unwrap();

                                                if let Some(0) = post_res.exit_code {
                                                    // Tests pass now — run `cargo fmt` and notify success.
                                                    let _fmt_res = crate::cargo_command::execute_cargo_command("fmt", vec![], cwd_opt, true).await.unwrap();

                                                    let content = sacp::schema::ContentBlock::Text(TextContent::new("Cargo tests passed after automated fix and `cargo fmt` was run.".to_string()));
                                                    conn_cx.send_notification_to(ClientPeer, SessionNotification::new(prompt_req.session_id.clone(), SessionUpdate::UserMessageChunk(ContentChunk::new(content)))).unwrap();
                                                } else {
                                                    // Tests still failing — notify the client with the test output JSON.
                                                    let post_json = serde_json::to_string(&post_res).unwrap();
                                                    let fail_block = sacp::schema::ContentBlock::Text(TextContent::new(indoc::formatdoc! {
                                                        "Automated fix attempt completed, but tests still fail. Here is the test/build output (JSON):\n\n{post_json}"
                                                    }));
                                                    conn_cx.send_notification_to(ClientPeer, SessionNotification::new(prompt_req.session_id.clone(), SessionUpdate::UserMessageChunk(ContentChunk::new(fail_block)))).unwrap();
                                                }
                                            });

                                            Ok(())
                                        }
                                        _ => Ok(()),
                                    }
                                }
                            })
                    }
                },
                on_receive_request!(),
            )
            .serve(client)
            .await
    }
}
