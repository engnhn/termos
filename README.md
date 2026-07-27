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

install the binary:

```bash
curl -fsSL https://raw.githubusercontent.com/engnhn/termos/main/install.sh | bash
```

or build from source:

```bash
git clone https://github.com/engnhn/termos.git
cd termos
cargo build --release
sudo cp target/release/termos /usr/local/bin/
```

## configuration

configurations are stored in `~/.config/termos/connections.json` with strict `0600` permissions:

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
      }
    ]
  }
]
```

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
| `termos update` | fetch and build latest release from github |
| `termos usage` | open interactive terminal user manual |

## documentation

detailed documentation is in the `wiki/` directory.

## license

[mit](./LICENSE)
