<p align="center">
  <img src="./termos.png" alt="Termos Logo" width="220" />
</p>

# Termos

Termos is a secure, local-first SSH connection manager designed for developers who manage multiple virtual servers and require a fast, interactive command-line workspace. Built entirely in Rust with zero external runtime dependencies, it integrates directly with your system's native SSH client to establish transparent, signal-compliant terminal sessions.

### Interactive Terminal Dashboard

Unlike traditional connection managers that flood your terminal with text tables, running `termos list` launches an interactive terminal user interface. You can navigate your saved servers using arrow keys, execute an immediate SSH handshake by pressing Enter, add new credentials inline, or remove stale configurations with deletion confirmations. The interface automatically handles layout constraints, adapting to terminal resize events gracefully and keeping resources secure using panic-safe state guards.

### Zero-Dependency Credentials Handshake

Termos eliminates the need for external wrapper programs like `sshpass` by implementing a native `SSH_ASKPASS` handshake loop. When establishing a password-authenticated connection, it transmits the credentials through standard environment buffers. This prevents passwords from leaking into system process trees, keeping your access keys private and secure.

### System Permissions & Local-First Philosophy

Security is enforced at the kernel boundary. Termos stores all configuration profiles locally in standard configuration paths with strict user-only read and write permissions. Your servers, usernames, ports, and authentication details never leave your machine and are inaccessible to other users sharing the host.

### Installation and Setup

Deploying Termos globally is handled by a single automated installer. Running the script builds the optimized binary and configures shell completions for your active terminal environment. 

```bash
chmod +x install.sh
./install.sh
```

Once installed, simply open a new terminal window to begin. You can invoke the interactive manager directly, launch explicit hosts, or utilize tab autocompletions for subcommands and registered server nicknames.
