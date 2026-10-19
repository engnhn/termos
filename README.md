<p align="center">
  <img src="./termos.png" alt="Termos logo" width="120" />
</p>

<p align="center">
  <a href="https://github.com/engnhn/termos/actions/workflows/ci.yml"><img src="https://github.com/engnhn/termos/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/engnhn/termos/releases"><img src="https://img.shields.io/github/v/release/engnhn/termos" alt="Latest release"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/github/license/engnhn/termos" alt="MIT license"></a>
</p>

# Termos

An SSH connection manager for Linux, with a keyboard-driven terminal interface and a local MCP server.

Save your hosts once, find them in the dashboard, and connect with your system's `ssh` client. Termos can also expose saved hosts and quick commands to an MCP client such as Antigravity.

## What you can do

- Save SSH hosts and organize them with groups.
- Search hosts and connect from the terminal dashboard.
- Save named commands for routine server tasks.
- Use the same saved hosts and commands from an MCP client over local stdio.
- Keep connection details in your user config and passwords in the Linux Secret Service.

Quick commands run with the permissions of the configured SSH account. Review a command before saving or running it.

## Install

Verified release binaries currently support **Linux x86_64**. The installer uses GitHub CLI 2.102.0 or newer to verify the release artifact's provenance.

```bash
git clone https://github.com/engnhn/termos.git
cd termos
./install.sh
```

The installer places `termos` in `~/.local/bin` (or `$XDG_BIN_HOME`). Add that directory to `PATH` if needed. To install manually, download the `termos-linux-x86_64` asset from [Releases](https://github.com/engnhn/termos/releases), verify it with `gh attestation verify`, then place it in a directory on your `PATH`.

To build from source, install Rust and run:

```bash
cargo build --release
install -Dm755 target/release/termos "$HOME/.local/bin/termos"
```

## First run

Start Termos, press `a` to add a host, then select it and press `Enter` to connect.

```bash
termos
```

Use `/` to search, `g` to filter by group, `c` to choose a saved quick command, and `q` to exit. The [TUI guide](wiki/TUI-and-Interactive-Controls.md) lists the controls.

## MCP client setup

Termos runs as a local stdio server. Add the installed binary to your MCP client's configuration:

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

For Antigravity, use `~/.gemini/config/mcp_config.json`, then inspect the loaded server under **Additional Options → MCP Servers**. Restart Antigravity if it does not appear.

The default MCP tools list saved hosts and quick commands, save a quick command when asked, and run a named saved command. A command call can open a one-shot SSH connection, or reuse a persistent session created with `connect_server`; disconnect it with `disconnect_server` when finished. Termos does not accept arbitrary shell text by default. Add `--allow-remote-commands` to enable the separate `run_ssh_command` tool.

MCP commands run with the saved SSH account's permissions and may change remote systems. Ask the MCP client to request approval before running commands. See the [MCP and CLI guide](wiki/CLI-Reference-and-Operations.md) for tool behavior, timeouts, and session details.

## Data and credentials

Connection profiles are stored in `~/.config/termos/connections.json`. Passwords are kept in the Linux Secret Service, such as GNOME Keyring or KWallet. Termos uses the system `ssh` client for connections. See [configuration](wiki/Configuration-Guide.md) and [security](wiki/Security-and-Credential-Isolation.md).

## Documentation

- [Getting started](wiki/Getting-Started.md)
- [CLI and MCP reference](wiki/CLI-Reference-and-Operations.md)
- [Configuration and quick commands](wiki/Configuration-Guide.md)
- [TUI controls](wiki/TUI-and-Interactive-Controls.md)
- [Security model](wiki/Security-and-Credential-Isolation.md)
- [Troubleshooting](wiki/Troubleshooting.md)
- [Architecture](wiki/Architecture-and-Design.md)

## Contributing

Bug reports and focused pull requests are welcome. Before opening a pull request, run `make check`. Please include the operating system, Termos version, and steps to reproduce a bug; remove hostnames, usernames, IP addresses, and other private details from logs.

## License

MIT. See [LICENSE](LICENSE).
