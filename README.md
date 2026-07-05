<p align="center">
  <img src="./termos.png" alt="Termos Logo" width="130" />
</p>

<h1 align="center">Termos</h1>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" /></a>
  <a href="WIKI.md"><img src="https://img.shields.io/badge/Interface-TUI%20Keyboard--Driven-blueviolet?style=for-the-badge" alt="TUI" /></a>
  <a href="WIKI.md"><img src="https://img.shields.io/badge/Security-Local--First-red?style=for-the-badge" alt="Security" /></a>
  <a href="https://www.linux.org"><img src="https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge&logo=linux" alt="Linux" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License" /></a>
</p>

---

Termos is a secure, local-first SSH connection manager designed for developers who manage multiple virtual servers and require a fast, interactive command-line workspace. Built entirely in Rust with zero external runtime dependencies, it integrates directly with your system's native SSH client to establish transparent, signal-compliant terminal sessions.

For detailed architecture, configuration schema, and advanced usage, refer to the [Termos Wiki](WIKI.md).

---

## Key Pillars & Philosophy

### Keyboard-Driven Terminal UI & Dashboard Navigation
Unlike traditional connection managers that flood your terminal with verbose text tables, running `termos list` launches a high-fidelity, interactive terminal user interface. You can navigate your registered connections using arrow keys, execute an immediate SSH handshake by pressing `Enter`, add new configurations inline with `a`, or safely remove entries with deletion confirmations using `d`. The TUI automatically handles terminal layout constraints, adapting gracefully to resize events, and leverages RAII state guards (`TerminalGuard`) to ensure the raw terminal state is fully restored under all exit conditions.

### Zero-Dependency Credential Isolation & Kernel-Bound Security
Security is enforced at the system boundary. Termos eliminates the need for wrapper programs like `sshpass` by implementing a native `SSH_ASKPASS` handshake loop. When establishing password-authenticated connections, it transmits credentials securely through standard environment buffers, preventing passwords from leaking into system process trees. 

All configuration profiles are stored locally in standard platform configuration directories (e.g., `~/.config/termos/connections.json`) under strict `0600` user-only permissions. Your servers, credentials, and settings never leave your machine, remaining inaccessible to other users sharing the host.

---

## Automated Global Installation & Shell Completions

Deploying Termos globally is handled by a single automated installer. Running the script builds the optimized release binary, registers it to your PATH, and configures autocomplete completions for Bash and Zsh.

```bash
# Install directly from GitHub
curl -fsSL https://raw.githubusercontent.com/engnhn/termos/main/install.sh | bash

# Or install locally from source
./install.sh
```

Once installed, simply open a new shell to begin. You can invoke the interactive manager directly, launch explicit connections, or utilize tab autocompletions for subcommands and registered server nicknames.
