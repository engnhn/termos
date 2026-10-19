# security and credential isolation

Passwords are stored in the operating system credential store (Linux Secret Service, such as GNOME Keyring or KWallet). The JSON file contains only an opaque UUID reference. Legacy JSON passwords are migrated on read; if the credential store cannot save a password, the migration stops and leaves the original JSON intact. Passwords are not encrypted by Termos itself, and an unlocked keyring or a compromised process running as the same user can access them.

On Unix, password-authenticated SSH uses the native `ssh_askpass` mechanism. Termos writes the password to an unlinked temporary file opened with mode `0600`, then passes its inherited file descriptor and byte length to `ssh` and the askpass callback. The password is not placed in process arguments or environment variables. The descriptor remains accessible to the SSH process tree for the session, so this is protection against routine environment inspection, not a boundary against the same user.

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
    │   │ (inherited anonymous credential file)│   │
    │   └──────────────────────────────────────┘   │
    └──────────────────────────────────────────────┘
```

terminal integrity is protected using an raii guard structure (`terminalguard`). raw terminal mode, alternate screen buffers, and hidden cursor states can leave host shells unresponsive if an application panics or exits unexpectedly. `terminalguard` implements the rust `drop` trait, guaranteeing cleanup execution under all exit conditions.

| operational phase | security control | technical mechanism |
|---|---|---|
| at-rest credential storage | OS credential store | Secret Service stores the password; JSON has only a UUID reference |
| credential transport | `ssh_askpass` callback | reads from an unlinked mode-0600 file descriptor |
| process exposure | no password environment variable | passes descriptor metadata only; same-user process access remains possible |
| terminal state | `terminalguard` raii | disables raw mode and restores screen buffer on drop |
| storage security | `0600` permissions | restricts configuration file access strictly to owning user |

Connection metadata is stored locally in standard user configuration directories. The configuration directory is set to `0700`, and each saved file is created with `0600` permissions. SSH password authentication is disabled on non-Unix platforms rather than falling back to an environment variable.

Release binaries are accepted only after `gh attestation verify` validates the binary's provenance for this repository and the pinned release workflow. The release workflow builds tagged source and publishes the Linux x86_64 binary with an artifact attestation. `gh release verify` checks a separate release-level attestation; Termos does not depend on it. Updates fail closed for binaries without a matching artifact attestation. This proves which workflow produced the binary, not that the source is free of vulnerabilities; repository branch and tag protections remain part of the trust model.

---

previous: [tui and interactive controls](./TUI-and-Interactive-Controls.md) | home: [home](./Home.md) | next: [configuration guide](./Configuration-Guide.md)
