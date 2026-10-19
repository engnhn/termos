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

## documentation

detailed documentation is in the `wiki/` directory.

## license

[mit](./LICENSE)
