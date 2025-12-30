# symposium-cargo

An ACP proxy wrapping a MCP server for executing Cargo commands with structured JSON output.

Part of the [Symposium](https://github.com/symposium-dev) project.

## Features

- Execute cargo commands (`check`, `build`, etc.) via MCP tools
- Structured JSON output with filtered compiler messages
- Automatic stderr filtering (e.g. removes file lock messages)

## Usage

The server provides MCP tools for common cargo operations:

- `cargo_check` - Run `cargo check`
- `cargo_build` - Run `cargo build`
- `cargo_test` - Run `cargo test` with optional test name/pattern

### Response Format

```json
{
  "exit_code": 0,
  "messages": [
    "warning: unused variable",
    "error: unexpected semicolon",
  ],
  "stderr": "Checking project v0.1.0",
  "command": "cargo check --message-format json",
  "build_success": true
}
```

## Integration

This server integrates with the Symposium Agent Client Protocol (SACP) framework.
