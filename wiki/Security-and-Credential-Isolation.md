# security and credential isolation

termos enforces security at system and process boundaries by avoiding external helper utilities like `sshpass` and eliminating plaintext password exposure in process argument lists. when establishing password-authenticated connections, termos utilizes a native `ssh_askpass` interceptor loop.

during connection initialization, termos exports `ssh_askpass` environment variables pointing to its own binary executable while staging password payload bytes in a protected environment variable (`termos_askpass_password`). when the native `ssh` client requests user credentials, it invokes the binary callback. termos outputs the staged password string directly to standard output into ssh internal memory and immediately clears environment variables post-handshake, preventing credential leakage in process inspection utilities like `ps` or `/proc`.

```
┌────────────────────────────────────────────────────────┐
│                        termos                          │
│                                                        │
│   ┌─────────────────────┐       ┌──────────────────┐   │
│   │    terminalguard    │◄──────│   crossterm tui  │   │
│   │  (raii raw restore) │       │   (user input)   │   │
│   └──────────┬──────────┘       └────────┬─────────┘   │
└──────────────┼───────────────────────────┼─────────────┘
               │                           │
               ▼ spawns subprocess         ▼
    ┌──────────────────────────────────────────────┐
    │                 ssh client                   │
    │                                              │
    │   ┌──────────────────────────────────────┐   │
    │   │ ssh_askpass interceptor loop         │   │
    │   │ (secure credential injection pipe)   │   │
    │   └──────────────────────────────────────┘   │
    └──────────────────────────────────────────────┘
```

terminal integrity is protected using an raii guard structure (`terminalguard`). raw terminal mode, alternate screen buffers, and hidden cursor states can leave host shells unresponsive if an application panics or exits unexpectedly. `terminalguard` implements the rust `drop` trait, guaranteeing cleanup execution under all exit conditions.

| operational phase | security control | technical mechanism |
|---|---|---|
| credential transport | `ssh_askpass` loop | pipes password via direct stdout stream to native ssh client |
| process isolation | memory clearing | unsets environment variables immediately after handshake |
| terminal state | `terminalguard` raii | disables raw mode and restores screen buffer on drop |
| storage security | `0600` permissions | restricts configuration file access strictly to owning user |

configurations are stored locally in standard user configuration directories. file initialization enforces strict `0600` read and write permissions, preventing access from unauthorized local host users. credential payloads remain entirely local and are never transmitted to external endpoints.

---

previous: [tui and interactive controls](./TUI-and-Interactive-Controls.md) | home: [home](./Home.md) | next: [configuration guide](./Configuration-Guide.md)
