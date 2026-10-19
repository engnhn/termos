# Configuration and quick commands

Termos keeps connection profiles in `~/.config/termos/connections.json`. The directory is created with mode `0700` and the file with mode `0600` on Unix. You can edit this file directly, but the dashboard and `termos add` wizard are safer ways to avoid malformed JSON.

## Profile format

```json
[
  {
    "nickname": "web-01",
    "host": "192.0.2.10",
    "port": 22,
    "username": "deploy",
    "ssh_key": "/home/you/.ssh/id_ed25519",
    "group": "web",
    "quick_commands": [
      { "name": "disk usage", "command": "df -h" }
    ]
  }
]
```

| Field | Meaning |
|---|---|
| `nickname` | Local name used by the dashboard and CLI |
| `host` | Hostname or IP address passed to SSH |
| `port` | SSH port; use `22` when not specified by your server |
| `username` | Remote account name |
| `ssh_key` | Optional path to a private key; SSH still handles key use and passphrases |
| `group` | Optional label for dashboard filtering |
| `quick_commands` | Optional named shell commands to run on this host |

Passwords are not stored as a profile field in current files. They are saved in the desktop Secret Service and referenced separately. Older files containing a plaintext password are migrated when read. If the Secret Service cannot save the credential, migration stops and the original file is kept.

## Quick commands

A quick command has a name and command string. Add, list, edit, or delete them with `termos qc`; omit optional details to use the interactive manager.

```bash
termos qc add web-01 --name "service status" --cmd "systemctl status nginx"
termos qc list web-01
termos connect web-01 --qc "service status"
```

Commands execute remotely as the configured SSH user. A saved command can modify or remove remote data. Review it before use, especially when an MCP client can invoke it.

See [CLI and MCP](CLI-Reference-and-Operations.md) for the complete command list and MCP behavior.
