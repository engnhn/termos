# architecture and design

termos operates as a single static binary compiled with rust. it acts as a lightweight orchestration layer between the user terminal and the native system ssh client, delegating session management, terminal emulation, and socket encryption directly to `ssh`.

the default `termos` command and `termos list` open the interactive connection dashboard. selecting a server restores the terminal before starting SSH. on Unix, interactive SSH replaces the Termos process; quick commands run SSH as a child process and return to the dashboard when it exits.

| metric | typical value | operational detail |
|---|---|---|
| binary size | ~3.5 mb | static release binary compiled with rust |
| memory usage | ~2-4 mb rss | resident memory footprint during active tui execution |
| cpu overhead | < 0.05% | event-driven input handling with zero background polling |
| runtime dependencies | `ssh`, `gh` for verified updates, desktop Secret Service for saved passwords | SSH itself remains the system native client |

the tui layout engine enforces viewport minimum boundaries to prevent text wrapping and component clipping. the main connections dashboard requires a viewport of at least 76 columns by 18 rows, while the quick command manager requires at least 60 columns by 14 rows. if window dimensions drop below required bounds, termos halts layout rendering, displays a centered warning banner, and resumes interface drawing automatically when the terminal is resized above the threshold.

---

previous: [configuration guide](./Configuration-Guide.md) | home: [home](./Home.md) | next: [cli reference and operations](./CLI-Reference-and-Operations.md)
