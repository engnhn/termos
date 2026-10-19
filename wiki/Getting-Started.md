# Getting started

## Requirements

- Linux x86_64 for the verified release binary.
- GitHub CLI 2.102.0 or newer to verify and install that binary.
- The system `ssh` client.
- A Linux Secret Service, such as GNOME Keyring or KWallet, if you save a password in Termos.

## Install the latest release

```bash
git clone https://github.com/engnhn/termos.git
cd termos
./install.sh
```

The installer verifies the downloaded binary's GitHub artifact attestation before copying it to `~/.local/bin` (or `$XDG_BIN_HOME`). Add the install directory to `PATH` if the installer reports that it is missing.

## Add a connection

Run `termos`, press `a`, and fill in the host, port, username, and optional key path or password. Save the profile, select it, and press `Enter` to connect. Termos starts the system SSH client; your normal SSH configuration and host key checks still apply.

Press `q` to leave the dashboard. Use `/` to search profiles, `g` to filter by group, and `c` to run a saved quick command. The [TUI guide](TUI-and-Interactive-Controls.md) has the full key reference.

## Add a quick command

Select a host, press `e`, and open its quick-command editor, or use the CLI:

```bash
termos qc add web-01 --name "disk usage" --cmd "df -h"
termos connect web-01 --qc "disk usage"
```

Quick commands are shell text executed on the selected remote host with that SSH account's permissions. Only save commands you trust.

## Use with an MCP client

Add the installed binary to the client's MCP server configuration. The [CLI and MCP guide](CLI-Reference-and-Operations.md) includes an Antigravity example and explains one-shot and persistent SSH sessions.
