# configuration guide

termos persists connection profiles and quick commands in a single json configuration file located in the user platform configuration directory `~/.config/termos/connections.json`. the file is created automatically on first save with strict `0600` permissions.

```json
[
  {
    "nickname": "prod-web-01",
    "host": "192.168.1.15",
    "port": 22,
    "username": "root",
    "password": "sample-password",
    "ssh_key": "/home/user/.ssh/id_ed25519",
    "group": "production",
    "quick_commands": [
      {
        "name": "disk usage",
        "command": "df -h"
      },
      {
        "name": "service status",
        "command": "systemctl status nginx"
      }
    ]
  }
]
```

the json schema represents an array of host objects. each field configures specific connection parameters used during ssh process instantiation and dashboard rendering.

| field | type | requirement | description |
|---|---|---|---|
| `nickname` | string | required | unique identifier used for cli connections and autocomplete |
| `host` | string | required | hostname or ip address of the remote server |
| `port` | integer | required | ssh port number (defaults to 22 if unassigned) |
| `username` | string | required | remote user account name for authentication |
| `password` | string | optional | plaintext password payload used by `ssh_askpass` loop |
| `ssh_key` | string | optional | absolute filesystem path to private ssh key file |
| `group` | string | optional | tag used for filtering hosts in the dashboard view |
| `quick_commands` | array | optional | list of script objects (`name`, `command`) attached to host |

configuration parameters can be edited interactively through the tui wizard (`e` key on selected host) or modified by editing `connections.json` directly. when editing manually, ensure valid json syntax and verify file permissions remain restricted to user read/write (`chmod 600 ~/.config/termos/connections.json`).

---

previous: [security and credential isolation](./Security-and-Credential-Isolation.md) | home: [home](./Home.md) | next: [architecture and design](./Architecture-and-Design.md)
