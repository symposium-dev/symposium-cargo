use anyhow::Result;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use tokio::process::Command;

/// Filter out cargo file lock messages from stderr
fn filter_stderr(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| !line.contains("Blocking waiting for file lock on package cache"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parameters for cargo command execution
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CargoCommandParams {
    /// The cargo command to execute (e.g., "check", "build", "test")
    pub command: String,
    /// Optional arguments to pass to the cargo command
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional working directory (defaults to current directory)
    pub cwd: Option<String>,
}

/// Result of cargo command execution
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CargoCommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

/// Execute a cargo command using tokio's async process
pub async fn execute_cargo_command_async(
    params: CargoCommandParams,
) -> Result<CargoCommandResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg(&params.command);
    cmd.args(&params.args);
    
    if let Some(cwd) = &params.cwd {
        cmd.current_dir(cwd);
    }
    
    let output = cmd.output().await?;
    
    Ok(CargoCommandResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: filter_stderr(&String::from_utf8_lossy(&output.stderr)),
        command: format!("cargo {} {}", params.command, params.args.join(" ")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cargo_version() {
        let params = CargoCommandParams {
            command: "version".to_string(),
            args: vec![],
            cwd: None,
        };
        
        let result = execute_cargo_command_async(params).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("cargo"));
    }

    #[tokio::test]
    async fn test_cargo_with_args() {
        let params = CargoCommandParams {
            command: "help".to_string(),
            args: vec!["build".to_string()],
            cwd: None,
        };
        
        let result = execute_cargo_command_async(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.command, "cargo help build");
    }
}
