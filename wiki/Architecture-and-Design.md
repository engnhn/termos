# architecture and design

termos operates as a single static binary compiled with rust. it acts as a lightweight orchestration layer between the user terminal and the native system ssh client, delegating session management, terminal emulation, and socket encryption directly to `ssh`.

the default `termos` command and `termos list` open the interactive connection dashboard. selecting a server restores the terminal before starting SSH. on Unix, interactive SSH replaces the Termos process; quick commands run SSH as a child process and return to the dashboard when it exits.

`termos mcp` runs the official Rust MCP SDK on stdio, keeping protocol traffic on stdout and diagnostics on stderr. By default it lists saved servers and quick commands, then runs only a named quick command stored for a saved server. Commands use one-shot SSH connections unless the client first calls `connect_server` and passes the returned ID to related operations. Persistent connections are owned by the MCP process, use a private ControlMaster socket directory, and close on `disconnect_server` or process exit. Up to four sessions and four concurrent SSH commands are allowed. `--allow-remote-commands` separately opts into arbitrary shell text. Passwords and private key contents are never returned; password bytes staged for SSH are wiped after master authentication. MCP command execution captures stdout and stderr with a 32 KiB limit per stream, escapes terminal controls, and enforces a 1–120 second timeout (30 seconds by default). A timeout kills the local SSH process group but cannot guarantee termination of a command detached remotely. Command tools advertise their destructive behavior using advisory MCP annotations.

| metric | typical value | operational detail |
|---|---|---|
| binary size | ~3.5 mb | static release binary compiled with rust |
| memory usage | ~2-4 mb rss | resident memory footprint during active tui execution |
| cpu overhead | < 0.05% | event-driven input handling with zero background polling |
| runtime dependencies | `ssh`, `gh` for verified updates, desktop Secret Service for saved passwords, MCP client for `termos mcp` | SSH itself remains the system native client |

the tui layout engine enforces viewport minimum boundaries to prevent text wrapping and component clipping. the main connections dashboard requires a viewport of at least 76 columns by 18 rows, while the quick command manager requires at least 60 columns by 14 rows. if window dimensions drop below required bounds, termos halts layout rendering, displays a centered warning banner, and resumes interface drawing automatically when the terminal is resized above the threshold.

---

previous: [configuration guide](./Configuration-Guide.md) | home: [home](./Home.md) | next: [cli reference and operations](./CLI-Reference-and-Operations.md)
