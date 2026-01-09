mod cargo_command;
pub mod cargo_mcp;

use std::sync::Arc;

use anyhow::Result;
pub use cargo_mcp::build_mcp_server;
use sacp::component::Component;
use sacp::link::{ConductorToProxy, ProxyToConductor};
use sacp::schema::{
    PromptRequest, SessionNotification, SessionUpdate, TextContent, ToolCallStatus,
};
use sacp::{AgentPeer, ClientPeer, on_receive_request};
use tokio::sync::{Mutex, RwLock};

pub struct CargoProxy;

impl Component<ProxyToConductor> for CargoProxy {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        let cwd = Arc::new(RwLock::new(None));
        let has_unchecked_changes_to_rs_files = Arc::new(Mutex::new(false));
        ProxyToConductor::builder()
            .name("cargo-proxy")
            .with_mcp_server(build_mcp_server(cwd.clone()))
            .on_receive_request_from(
                ClientPeer,
                {
                    let cwd = cwd.clone();
                    let has_unchecked_changes_to_rs_files = has_unchecked_changes_to_rs_files.clone();
                    async move |prompt_req: PromptRequest, req_cx, conn_cx| {
                        conn_cx
                            .send_request_to(AgentPeer, prompt_req.clone())
                            .on_receiving_ok_result(req_cx, {
                                let cwd = cwd.clone();
                                let has_unchecked_changes_to_rs_files = has_unchecked_changes_to_rs_files.clone();
                                move |res, req_cx| async move {
                                    req_cx.respond(res.clone())?;
                                    match res.stop_reason {
                                        sacp::schema::StopReason::EndTurn => {
                                            let mut has_unchecked_changes_to_rs_files = has_unchecked_changes_to_rs_files.lock().await;
                                            tracing::debug!(has_unchecked_changes_to_rs_files = ?*has_unchecked_changes_to_rs_files);
                                            if !*has_unchecked_changes_to_rs_files {
                                                return Ok(());
                                            }
                                            *has_unchecked_changes_to_rs_files = false;
                                            drop(has_unchecked_changes_to_rs_files);
                                            let cwd = cwd.read().await.clone();

                                            let res = crate::cargo_command::execute_cargo_command("check", vec![], cwd, false).await?;
                                            if let Some(0) = res.exit_code {
                                                return Ok(());
                                            }
                                            let json = serde_json::to_string(&res)?;
                                            let content = sacp::schema::ContentBlock::Text(TextContent::new(indoc::formatdoc! {"
                                                Cargo check has automatically been run and the project failed to build with the following output. Please fix the errors.

                                                {json}
                                            "}));
                                            conn_cx.send_request_to(AgentPeer, PromptRequest::new(prompt_req.session_id, vec![content]));

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
            .on_receive_notification_from(
                AgentPeer,
                {
                    let has_unchecked_changes_to_rs_files = has_unchecked_changes_to_rs_files.clone();
                    async move |notification: SessionNotification, cx| {
                        if let SessionUpdate::ToolCallUpdate(update) = &notification.update
                            && update
                                .fields
                                .status
                                .map(|s| s == ToolCallStatus::Completed)
                                .unwrap_or(false)
                            && update
                                .fields
                                .locations
                                .as_ref()
                                .map(|l| {
                                    l.iter().any(|l| {
                                        l.path
                                            .extension()
                                            .map(|e| e.eq_ignore_ascii_case("rs"))
                                            .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false)
                        {
                            *has_unchecked_changes_to_rs_files.lock().await = true;
                        }

                        cx.send_notification_to(ClientPeer, notification)?;
                        Ok(())
                    }
                },
                sacp::on_receive_notification!(),
            )
            .serve(client)
            .await
    }
}
