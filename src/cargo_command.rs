use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Filter cargo JSON messages to keep only compiler-message and build-finished
fn filter_json_messages(stdout: &str) -> Vec<serde_json::Value> {
    stdout
        .lines()
        .filter_map(|line| {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(reason) = json.get("reason").and_then(|r| r.as_str()) {
                    if reason == "compiler-message" || reason == "build-finished" {
                        let mut filtered = serde_json::Map::new();
                        json.get("reason")
                            .map(|s| filtered.insert("reason".to_string(), s.clone()));
                        json.get("package_id")
                            .map(|s| filtered.insert("package_id".to_string(), s.clone()));
                        json.get("success")
                            .map(|s| filtered.insert("success".to_string(), s.clone()));
                        json.get("message")
                            .and_then(|m| m.get("rendered"))
                            .map(|s| filtered.insert("rendered_message".to_string(), s.clone()));
                        return Some(serde_json::Value::Object(filtered));
                    }
                }
            }
            None
        })
        .collect()
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
    pub success: bool,
    pub exit_code: Option<i32>,
    pub messages: Vec<serde_json::Value>,
    pub stderr: String,
    pub command: String,
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

    Ok(CargoCommandJsonResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        messages: filter_json_messages(&String::from_utf8_lossy(&output.stdout)),
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
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_cargo_with_args() {
        let result = execute_cargo_command("help", vec!["build"], None, true)
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.command.contains("cargo help build"));
    }
}
