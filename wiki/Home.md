# termos

termos is a lightweight local ssh connection manager written in rust. it manages saved host configurations, executes interactive ssh sessions using native system binaries, isolates credentials in process memory, and provides terminal keyboard controls for navigation, search, and quick command execution.

## index

| page | summary |
|---|---|
| [tui and interactive controls](./TUI-and-Interactive-Controls.md) | dashboard navigation, server wizard input manipulation, search-as-you-type filtering, group popovers, and quick command submenus |
| [security and credential isolation](./Security-and-Credential-Isolation.md) | native `ssh_askpass` credential injection loop, process memory clearing, `terminalguard` raii state recovery, and file permissions |
| [configuration guide](./Configuration-Guide.md) | json storage layout, field definitions, permission enforcement, quick command structures, and file resolution paths |
| [architecture and design](./Architecture-and-Design.md) | static binary architecture, native subprocess spawning, tui suspension and resumption lifecycle, and layout viewport bounds |
| [cli reference and operations](./CLI-Reference-and-Operations.md) | command line execution syntax, subcommand reference table, quick command flags, build workflow, and shell completions |

---

home: [home](./Home.md) | next: [tui and interactive controls](./TUI-and-Interactive-Controls.md)
