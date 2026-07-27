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
| `termos update` | none | downloads and compiles latest release version from github |
| `termos usage` | none | opens interactive page-based terminal user manual |

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
