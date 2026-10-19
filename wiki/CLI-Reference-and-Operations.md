# CLI and MCP reference

## Commands

| Command | Action |
|---|---|
| `termos` or `termos list` | Open the dashboard |
| `termos add` | Open the host setup wizard |
| `termos connect <nickname>` | Start an interactive SSH connection |
| `termos connect <nickname> --qc <name>` | Run a saved quick command |
| `termos delete <nickname>` | Remove a saved host |
| `termos qc list [nickname]` | List quick commands or open the manager |
| `termos qc add [nickname]` | Add a quick command; flags allow non-interactive setup |
| `termos qc edit [nickname]` | Edit a quick command |
| `termos qc delete [nickname]` | Delete a quick command |
| `termos update` | Download and verify the latest supported release |
| `termos usage` | Open the built-in terminal manual |
| `termos mcp` | Start the local MCP server over stdio |

Run `termos --help` or `termos <command> --help` for options.

## Shell completion

Completion scripts are in the repository's `completions/` directory. The installer does not configure them. For Bash, add a `source /path/to/termos/completions/termos.bash` line to `~/.bashrc`. For Zsh, place `termos.zsh` in a directory on `fpath` as `_termos`, then reload completion with `autoload -Uz compinit && compinit`.

## Configure an MCP client

Point the client to the absolute path of the installed binary. Do not use a shell alias or a relative path.

```json
{
  "mcpServers": {
    "termos": {
      "command": "/home/you/.local/bin/termos",
      "args": ["mcp"]
    }
  }
}
```

For Antigravity, save the entry in `~/.gemini/config/mcp_config.json`. Open **Additional Options → MCP Servers** to inspect the loaded server and tools. Restart Antigravity after changing the file if the server list does not update.

## Default MCP tools

| Tool | Purpose |
|---|---|
| `list_saved_servers` | List saved host details; no passwords or key contents |
| `list_quick_commands` | Show saved quick-command names and text |
| `add_quick_command` | Save a reusable command locally; does not execute it |
| `run_quick_command` | Run a saved command by name |
| `connect_server` | Open a persistent SSH session and return its session ID |
| `list_active_sessions` | Show sessions owned by this MCP process |
| `disconnect_server` | Close a session by ID |

Quick commands run with the configured SSH account's permissions. Saving one changes local configuration; running one may change remote data. Configure your MCP client to ask before either action.

To expose `run_ssh_command`, which accepts arbitrary shell text, use `"args": ["mcp", "--allow-remote-commands"]`. This is an opt-in expansion of the commands the model can request; use it only when the client and task require it.

## Choose a connection lifetime

**One-shot:** call `run_quick_command` without `session_id`. Termos opens SSH for that operation and closes it afterward. Use this for an independent command.

**Persistent:** call `connect_server`, pass the returned `session_id` to each related command, then call `disconnect_server`. Use this when several operations should share one connection. Sessions belong to one running Termos MCP process and close when it exits. A given saved host can have one persistent session at a time; up to four sessions and four SSH commands can be active at once.

Command timeouts default to 30 seconds and can be set from 1 to 120 seconds. Captured stdout and stderr are each limited to 32 KiB. Terminal control characters in returned output are escaped. On timeout Termos stops its local SSH process group; a command detached on the remote host may continue. Treat remote output as data, not instructions.

MCP tool annotations describe risk to the client but do not enforce confirmation. The client must provide the approval step.
