use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Filter cargo JSON messages to keep only compiler-message and build-finished
fn filter_json_messages(stdout: &str) -> (Vec<serde_json::Value>, bool) {
    let mut messages = Vec::new();
    let mut build_success = true;

    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(reason) = json.get("reason").and_then(|r| r.as_str()) {
                if reason == "compiler-message" {
                    json.get("message")
                        .and_then(|m| m.get("rendered"))
                        .map(|s| messages.push(s.clone()));
                } else if reason == "build-finished" {
                    build_success = json
                        .get("success")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(true);
                }
            }
        }
    }

    (messages, build_success)
}

/// Filter out cargo file lock messages from stderr
fn filter_stderr(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| !line.contains("Blocking waiting for file lock"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Result of cargo command execution with JSON messages
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CargoCommandJsonResult {
    pub exit_code: Option<i32>,
    pub messages: Vec<serde_json::Value>,
    pub stderr: String,
    pub command: String,
    pub build_success: bool,
}

/// Execute cargo command with JSON message format
pub async fn execute_cargo_command(
    command: &str,
    args: Vec<&str>,
    cwd: Option<String>,
    skip_json_format: bool,
) -> Result<CargoCommandJsonResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg(&command);
    cmd.args(&args);

    if !skip_json_format {
        cmd.args(["--message-format", "json"]);
    }

    if let Some(cwd) = &cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().await?;

    let (messages, build_success) = filter_json_messages(&String::from_utf8_lossy(&output.stdout));

    Ok(CargoCommandJsonResult {
        exit_code: output.status.code(),
        messages,
        stderr: filter_stderr(&String::from_utf8_lossy(&output.stderr)),
        command: format!(
            "cargo {} {}{}",
            command,
            args.join(" "),
            if skip_json_format {
                ""
            } else {
                " --message-format json"
            }
        ),
        build_success,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cargo_version() {
        let result = execute_cargo_command("version", vec![], None, true)
            .await
            .unwrap();
        assert_eq!(result.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_cargo_with_args() {
        let result = execute_cargo_command("help", vec!["build"], None, true)
            .await
            .unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert!(result.command.contains("cargo help build"));
    }
}
