<p align="center">
  <img src="./termos.png" alt="termos logo" width="130" />
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/language-rust-orange?style=flat-square&logo=rust" alt="rust"></a>
  <a href="license"><img src="https://img.shields.io/badge/license-mit-blue.svg?style=flat-square" alt="license"></a>
  <a href="https://github.com/engnhn/termos/releases"><img src="https://img.shields.io/github/v/release/engnhn/termos?style=flat-square&color=emerald" alt="release"></a>
</p>

# termos

termos is a local ssh connection manager and terminal user interface for linux systems. it connects to remote hosts using the system native ssh binary and isolates stored credentials.

## quick start

Install GitHub CLI 2.102.0 or newer, then download and verify a release before installing it:

```bash
set -euo pipefail
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
gh release download --repo engnhn/termos --pattern termos-linux-x86_64 --dir "$tmp_dir"
gh attestation verify "$tmp_dir/termos-linux-x86_64" --repo engnhn/termos --signer-workflow engnhn/termos/.github/workflows/release.yml
install -Dm755 "$tmp_dir/termos-linux-x86_64" "$HOME/.local/bin/termos"
```

Only releases published with the repository's signed-release workflow are accepted. Older releases without attestations fail verification and cannot be installed or updated automatically.

or build from source:

```bash
git clone https://github.com/engnhn/termos.git
cd termos
cargo build --release
install -Dm755 target/release/termos "$HOME/.local/bin/termos"
```

## configuration

Connection metadata is stored in `~/.config/termos/connections.json`; passwords are stored in the Linux desktop Secret Service (GNOME Keyring or KWallet). The config directory is `0700`, and the JSON file is `0600`.

```json
[
  {
    "nickname": "prod-web-01",
    "host": "192.168.1.15",
    "port": 22,
    "username": "root",
    "ssh_key": "/home/user/.ssh/id_ed25519",
    "group": "production",
    "quick_commands": [
      {
        "name": "disk usage",
        "command": "df -h"
      }
    ]
  }
]
```

Legacy plaintext passwords are moved to the system credential store when Termos next reads the connection file. That migration fails closed if a Secret Service is unavailable; the original JSON is retained until the credential has been stored successfully.

launch the interactive dashboard:

```bash
termos
```

## commands

| command | description |
|---|---|
| `termos` | launch interactive tui connection manager |
| `termos add` | launch interactive server registration wizard |
| `termos connect <nickname>` | establish direct ssh connection to host |
| `termos connect <nickname> -q <qc>` | execute pre-defined quick command on host |
| `termos delete <nickname>` | remove saved server configuration |
| `termos qc list [nickname]` | list quick commands for server |
| `termos qc add [nickname]` | add new quick command (interactive tui if flags omitted) |
| `termos qc edit [nickname]` | edit quick command (interactive tui if flags omitted) |
| `termos qc delete [nickname]` | delete quick command (interactive tui if flags omitted) |
| `termos update` | download and verify the latest signed release |
| `termos usage` | open interactive terminal user manual |
| `termos mcp` | run a local MCP server over stdio |

## MCP integration

Termos can expose saved SSH connections to local MCP clients over stdio. Add a server entry to the MCP client's configuration, using the absolute path to the installed binary:

```json
{
  "mcpServers": {
    "termos": {
      "command": "/home/user/.local/bin/termos",
      "args": ["mcp"]
    }
  }
}
```

For Antigravity, place the entry in `~/.gemini/config/mcp_config.json`. After saving it, open **Additional Options (…) → MCP Servers** to inspect the server and its tools; restart Antigravity if the server does not appear. Use the installed release binary path in `command`.

By default the server exposes saved-server and quick-command tools, plus `connect_server`, `list_active_sessions`, and `disconnect_server`. `add_quick_command` saves a reusable command to local Termos configuration but does not execute it; only save one when the user explicitly asks, because it can later run with the saved account's full permissions. Omit `session_id` from a command call for a one-shot SSH connection that closes after the command. For several related operations, call `connect_server` once, pass its `session_id` to each command, then call `disconnect_server`. Session IDs live only for that MCP process; sessions close when explicitly disconnected or when the process exits. Termos allows four active sessions and four simultaneous SSH commands. The default execution interface only runs commands previously saved in Termos; the model selects a server and quick-command name, never supplies arbitrary shell text. Quick commands run with the saved account's permissions and may modify remote systems, so configure Antigravity to request approval before command execution. To expose unrestricted `run_ssh_command`, add `--allow-remote-commands` to the MCP server arguments. Both command tools have a 30-second default timeout (1–120 seconds), cap stdout and stderr at 32 KiB each, and escape terminal control characters in returned output. A timeout kills the local SSH process group, though a command detached on the remote host may continue. Command tools are marked as potentially destructive, but MCP annotations are advisory.

To enable remote command execution, change the client entry to:

```json
"args": ["mcp", "--allow-remote-commands"]
```

## documentation

detailed documentation is in the `wiki/` directory.

## development

Run the same format, lint, and test checks used for pull requests with:

```bash
make check
```

Build an optimized release binary after those checks pass with `make release`. Release tags use the `v` prefix and must match the version in `Cargo.toml`; pushing a matching tag starts the signed Linux release workflow.

## license

[mit](./LICENSE)
