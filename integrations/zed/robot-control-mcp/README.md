# Robot Control MCP for Zed

This integration lets Zed launch the local `robot_control_mcp` stdio server.
It exposes the existing Robot Control MCP tools to Zed Agent without embedding
the desktop GUI in Zed.

## Binary resolution

Build the MCP binary first:

```powershell
cargo build -p robot_control_rust --bin robot_control_mcp
```

The dev extension resolves the MCP command in this order:

1. `ROBOT_CONTROL_MCP_PATH`, when set to an absolute binary path.
2. `robot_control_mcp` from `PATH`.

The binary must be launched with `--stdio`. It must not write logs to stdout.

## Custom context server

```powershell
cargo build -p robot_control_rust --bin robot_control_mcp
```

Add a custom context server in Zed settings:

```json
{
  "context_servers": {
    "robot-control-mcp": {
      "command": "C:\\path\\to\\rust_serial\\target\\debug\\robot_control_mcp.exe",
      "args": ["--stdio"]
    }
  }
}
```

On Linux or macOS, point `command` to the built `robot_control_mcp` binary and
keep the same `["--stdio"]` args.

## Dev extension

Install this directory as a Zed dev extension. If `robot_control_mcp` is not on
`PATH`, set `ROBOT_CONTROL_MCP_PATH` to the absolute binary path before starting
Zed.

Windows example:

```powershell
$env:ROBOT_CONTROL_MCP_PATH = "C:\path\to\rust_serial\target\debug\robot_control_mcp.exe"
zed .
```

Linux/macOS example:

```bash
export ROBOT_CONTROL_MCP_PATH="/path/to/rust_serial/target/debug/robot_control_mcp"
zed .
```

The extension only returns the MCP startup command. All tool behavior lives in
the main Rust project.

## Local smoke validation

Run the binary version check:

```powershell
.\target\debug\robot_control_mcp.exe --version
```

Run a stdio MCP smoke check:

```powershell
$exe = ".\target\debug\robot_control_mcp.exe"
$input = @(
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"zed-smoke","version":"0.0.0"}}}',
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}',
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}',
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_server_status","arguments":{}}}'
)
$input | & $exe --stdio
```

Expected results:

- stdout contains exactly three JSON-RPC responses.
- `tools/list` includes `get_server_status`.
- `tools/call get_server_status` reports `runtime_mode` as `stdio`.
- `hardware_write_enabled` is `false`.
- stderr is empty during the happy-path smoke check.

`set_pid_params` is intentionally limited to the MCP server's in-memory state.
It does not write serial, CAN, Modbus, USB, or other hardware outputs.

## Acceptance notes

This local integration intentionally does not download release assets or publish
to the Zed extension marketplace. Existing `cargo audit` maintenance warnings
and `cargo deny` duplicate/unescaper warnings are non-blocking for this MCP
closure and should be handled in a later dependency-upgrade batch.
