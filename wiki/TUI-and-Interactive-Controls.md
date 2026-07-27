# tui and interactive controls

termos runs an interactive terminal user interface built on crossterm. executing `termos` or `termos list` launches the main connections dashboard, rendering registered hosts within a bordered viewport. navigation relies strictly on keyboard input without requiring mouse interaction.

| key | action | operational context |
|---|---|---|
| `up` / `down` | select host | moves cursor highlight through listed connections |
| `enter` | connect / confirm | spawns interactive ssh session or confirms modal selection |
| `/` | search mode | activates top input bar for real-time search filtering |
| `esc` | clear / exit | clears query in search mode; exits dashboard otherwise |
| `a` | add server | opens full-screen server configuration wizard |
| `d` | delete server | prompts confirmation overlay to remove selected host |
| `e` | edit server | opens configuration manager menu for host fields or quick commands |
| `g` / `g` | group filter | opens selection popover listing unique group tags |
| `c` | quick commands | opens overlay listing executable remote script commands |
| `q` | exit | closes tui and restores host shell |

interactive input fields within the server wizard and quick command editor support character manipulation and cursor positioning. long text paths or commands scroll horizontally within fixed field boundaries using internal scroll offset tracking.

| key | action | operational context |
|---|---|---|
| `up` / `down` / `tab` / `backtab` | switch focus | moves input focus between form text fields and control buttons |
| `left` / `right` | move cursor | shifts text insertion point character-by-character |
| `home` / `end` | jump cursor | moves insertion point instantly to start or end of line |
| `backspace` / `delete` | remove character | deletes preceding character or character under cursor |
| `ctrl + s` | save entry | validates form input and persists configuration to storage |
| `esc` | cancel | discards uncommitted input and returns to main dashboard |

pressing `/` focuses the search bar at the top of the dashboard. as text is typed, the connection list filters dynamically across host nickname, hostname, username, and group. navigation arrows (`up` / `down`) remain active while typing, allowing instant host selection from filtered results. pressing `esc` clears active search queries and restores full list visibility.

pressing `g` opens a group selection overlay populated with all unique group strings present in storage. selecting a group isolates matching hosts in the viewport, while selecting `[show all]` resets active filters. remote scripts mapped to a server are executed by pressing `c`, selecting the target script, and pressing `enter`. the tui suspends during execution, displays command output in standard terminal stream mode, and resumes after key confirmation.

---

previous: [home](./Home.md) | home: [home](./Home.md) | next: [security and credential isolation](./Security-and-Credential-Isolation.md)
