# TUI controls

Run `termos` to open the host dashboard. It is keyboard-driven and uses the terminal's full-screen view.

## Dashboard

| Key | Action |
|---|---|
| `↑` / `↓` | Select a host |
| `Enter` | Connect to the selected host |
| `/` | Search by nickname, hostname, username, or group |
| `g` | Filter by group |
| `a` | Add a host |
| `e` | Edit the selected host or its quick commands |
| `c` | Choose a saved quick command |
| `d` | Delete the selected host after confirmation |
| `Esc` | Clear search or leave the current view |
| `q` | Exit the dashboard |

While searching, `←` / `→`, `Home`, `End`, `Backspace`, and `Delete` edit the query. Arrow keys can also move through the filtered host list.

## Forms

Use `Tab` and `Shift+Tab` or the arrow keys to move between fields and controls. `←` / `→` position the cursor; `Home` and `End` move to the start or end of the field. `Backspace` and `Delete` remove text. `Ctrl+S` saves a valid form. `Esc` cancels without saving.

Quick commands run on the remote host with the configured SSH account's permissions. Confirm the selected command before running it, especially if it changes remote state.

If a terminal is too small to render the current view, resize it and Termos will redraw when enough space is available.
