# Termos Technical Wiki & Reference Guide

This wiki serves as the authoritative guide for Termos configuration, CLI options, interactive TUI controls, database schema, and system architecture.

---

## 1. Interactive TUI Key Bindings

Termos is designed for rapid navigation without taking your hands off the keyboard.

### Main Connections Dashboard

| Key | Action | Context |
| :--- | :--- | :--- |
| `Up` / `Down` | Navigate through listed connections | Scrollbar updates automatically if list exceeds 8 rows |
| `Enter` | Connect to selected host | Spawns interactive SSH session |
| `a` | Add new server connection | Opens full-screen Add Wizard form |
| `d` | Delete connection configuration | Prompts confirmation banner |
| `e` | Manage server configuration | Opens Edit/Quick Command manager menu |
| `g` / `/` | Select group filter | Opens unique group list popover |
| `c` | Select & execute Quick Command | Opens quick commands popover for execution |
| `Esc` / `q` | Exit Termos dashboard | Returns safely to standard terminal shell |

### Quick Command TUI Manager (Fallback Mode)

| Key | Action | Context |
| :--- | :--- | :--- |
| `Up` / `Down` | Navigate through servers or quick commands | Dynamic scrolling |
| `Enter` | Confirm server selection / trigger action | Triggers Edit/Delete depending on current mode |
| `Esc` | Go back / Exit | Navigates back to server list or exits TUI |

---

## 2. Advanced Workflows: Grouping & Remote Execution

### Tag-Based Server Grouping
*   **Concept**: Organize your connections into distinct groups (e.g. `production`, `staging`, `testing`).
*   **Filtering**: Pressing `g` or `/` within the dashboard opens a selection overlay featuring all unique group tags.
*   **Behavior**: Selecting a group filters the visible server list, isolating keyboard focus and connectivity actions to the active namespace. Selecting `[All]` clears the filter.

### Quick Command Orchestration (`termos qc`)
*   **TUI Execution Submenu**: Pressing `c` on any server configuration loads its predefined quick commands. Selecting one executes the script over SSH, suspends the TUI, displays output in raw terminal view, and waits for a keypress confirmation before restoring the dashboard.
*   **Flexible CLI & TUI Management**: Run `termos qc [list|add|edit|delete] [nickname]` to manage quick commands. Omit options or names to drop into a premium interactive management TUI. Provide full flags (e.g. `termos qc add server --name "Usage" --cmd "df -h"`) to manage commands programmatically.

---

## 3. Command Line Interface (CLI) Specification

```bash
termos [COMMAND]
```

### Global Commands

*   `termos` or `termos list`: Launches the interactive TUI connection manager.
*   `termos add`: Launches the interactive server addition form.
*   `termos connect <nickname>`: Initiates a direct SSH connection to the registered host matching `<nickname>`.
*   `termos delete <nickname>`: Deletes the registered connection matching `<nickname>`.
*   `termos update`: Fetches and builds the latest release version directly from GitHub.

### Quick Command Subcommands (`termos qc`)

Manage command scripts mapped to specific servers. If parameters are omitted, Termos automatically launches the corresponding interactive TUI flow:

*   `termos qc list [nickname]`
*   `termos qc add [nickname] [--name <name> --cmd <command>]`
*   `termos qc edit [nickname] [--name <name> --new-name <new_name> --new-cmd <new_cmd>]`
*   `termos qc delete [nickname] [--name <name>]`

---

## 4. Storage Engine & Database Schema

All settings are persisted in JSON format in the user's config directory:
```bash
~/.config/termos/connections.json
```
This file is initialized with strict `0600` user-only read/write permissions to protect credentials.

### JSON Representation

```json
[
  {
    "nickname": "prod-web-01",
    "host": "192.168.1.15",
    "port": 22,
    "username": "root",
    "password": "secure-password",
    "ssh_key": "/home/user/.ssh/id_ed25519",
    "group": "production",
    "quick_commands": [
      {
        "name": "Disk Space",
        "command": "df -h"
      },
      {
        "name": "Service Status",
        "command": "systemctl status nginx"
      }
    ]
  }
]
```

---

## 5. Systems Architecture & Security Implementations

```
┌────────────────────────────────────────────────────────┐
│                        TERMOS                          │
│                                                        │
│   ┌─────────────────────┐       ┌──────────────────┐   │
│   │    TerminalGuard    │◄──────│   crossterm TUI  │   │
│   │  (RAII Raw Restore) │       │   (User Input)   │   │
│   └──────────┬──────────┘       └────────┬─────────┘   │
└──────────────┼───────────────────────────┼─────────────┘
               │                           │
               ▼ Spawns Subprocess         ▼
    ┌──────────────────────────────────────────────┐
    │                 ssh client                   │
    │                                              │
    │   ┌──────────────────────────────────────┐   │
    │   │ SSH_ASKPASS Interceptor Loop         │   │
    │   │ (Secure credential injection pipe)   │   │
    │   └──────────────────────────────────────┘   │
    └──────────────────────────────────────────────┘
```

### Password Isolation Loop
Termos does not read, write, or leak passwords to process listings. When password authentication is required:
1. It exports `SSH_ASKPASS` environment variables pointing to the running `termos` binary.
2. It sets `SSH_ASKPASS_PASSWORD` inside a secure process buffer.
3. The native SSH client calls back to `termos`, which pipes the credential to stdout directly into SSH's memory buffer.
4. The system variables are cleared instantly post-handshake, preventing any external read leak.

### RAII Terminal State Guarding
To prevent garbled shell sessions on crashes or unexpected exits, TUI actions are wrapped in the `TerminalGuard` structure. The guard implements the `Drop` trait, guaranteeing that:
- Alternate terminal screen buffers are closed.
- Terminal raw mode is disabled.
- Cursor visibility is restored.
