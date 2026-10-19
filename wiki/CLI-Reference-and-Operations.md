# cli reference and operations

termos commands follow the execution pattern `termos [subcommand] [arguments] [flags]`. running `termos` without arguments opens the interactive tui connections dashboard.

| subcommand | arguments & flags | description |
|---|---|---|
| `termos` | none | launches main interactive tui connection dashboard |
| `termos add` | none | launches full-screen interactive server configuration wizard |
| `termos connect` | `<nickname> [-q <qc_name>]` | connects to registered host or runs specified quick command |
| `termos delete` | `<nickname>` | removes registered host configuration matching nickname |
| `termos qc list` | `[nickname]` | lists quick commands for specified host or opens tui manager |
| `termos qc add` | `[nickname] [--name <name> --cmd <command>]` | adds quick command programmatically or via tui wizard |
| `termos qc edit` | `[nickname] [--name <name> --new-name <new_name> --new-cmd <new_cmd>]` | updates existing quick command fields |
| `termos qc delete` | `[nickname] [--name <name>]` | deletes quick command matching name |
| `termos update` | none | downloads the latest Linux x86_64 release and verifies its GitHub artifact attestation before installing |
| `termos usage` | none | opens interactive page-based terminal user manual |
| `termos mcp` | `[--allow-remote-commands]` | runs the local MCP server over stdio; saved quick commands are available by default, and the flag enables arbitrary SSH command execution |

## MCP client setup

Configure the MCP client to launch the installed Termos binary with the `mcp` argument. For clients using an `mcpServers` configuration, the entry looks like this:

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

For Antigravity, save this in `~/.gemini/config/mcp_config.json`. Inspect the loaded server and tools under **Additional Options (…) → MCP Servers**. Restart Antigravity if configuration changes do not appear.

By default Termos exposes saved-server and quick-command tools, plus `add_quick_command`, `connect_server`, `list_active_sessions`, and `disconnect_server`. `add_quick_command` writes a reusable command to local configuration but does not execute it. Use it only when the user explicitly asks to save one; saved commands later run with the server account's full permissions. Omit `session_id` from a command call for one-shot mode: Termos opens SSH for that command and closes it when the command ends. For multiple related operations, call `connect_server`, pass its returned `session_id` to each command, then call `disconnect_server`. Sessions belong to the running MCP process, close when disconnected or when it exits, and are limited to four at a time. Commands are limited to four concurrent SSH operations. The model can view and run commands saved in Termos, selected by server nickname and quick-command name. Configure the client to request approval before saving or running them. Add `--allow-remote-commands` to expose `run_ssh_command`, which accepts arbitrary shell text. Commands run with the saved account's permissions, time out after 30 seconds by default (maximum 120 seconds), return at most 32 KiB each of stdout and stderr. Terminal control characters are escaped in returned output. A timeout kills the local SSH process group but cannot guarantee termination of a command detached remotely. Passwords and private key contents are not exposed as tool results. Command tools are annotated as potentially destructive, but MCP annotations are advisory.

shell autocompletion scripts for bash and zsh are included in the `completions/` directory of the repository. during installation, `install.sh` registers these completions automatically. to configure completions manually, copy the corresponding completion script to system completion paths.

```bash
# bash completion
sudo cp completions/termos.bash /usr/share/bash-completion/completions/termos

# zsh completion
sudo cp completions/termos.zsh /usr/local/share/zsh/site-functions/_termos
```

restarting the shell or reloading shell configuration (`source ~/.bashrc` or `source ~/.zshrc`) enables tab completion for subcommands, options, and registered server nicknames.

---

previous: [architecture and design](./Architecture-and-Design.md) | home: [home](./Home.md) | next: [home](./Home.md)
