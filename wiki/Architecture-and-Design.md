# Architecture

Termos is a Rust command-line application with an interactive terminal interface. It stores profiles locally and starts the operating system's `ssh` executable for connections and remote commands.

## Process model

- Running `termos` opens the dashboard. Selecting a host starts an interactive SSH process after Termos restores the terminal state.
- A quick command runs SSH as a child process, displays its output, then returns control to the dashboard or shell.
- Running `termos mcp` starts an MCP server on stdin and stdout. Protocol output stays on stdout; errors are reported on stderr.
- MCP one-shot calls create an SSH connection for one operation. Persistent connections use OpenSSH multiplexing and are owned by the Termos MCP process that created them.

MCP connections are capped at four active sessions and four concurrent SSH commands. Captured command output is bounded to 32 KiB per stream. Interactive SSH runs in the user's terminal and does not use the MCP output capture limit.

## Local state

Connection metadata and named quick commands are stored in `~/.config/termos/connections.json`. Password credentials are stored through Linux Secret Service. See [Configuration](Configuration-Guide.md) and [Security](Security-and-Credential-Isolation.md) for their boundaries.

## Release process

Pushing a version tag that matches the Cargo package version starts the Linux x86_64 release workflow. It checks the tag is reachable from `main`, runs the repository's release checks, publishes the binary, and creates an artifact attestation. `termos update` verifies the downloaded artifact before installing it. Attestation records build provenance; users should still review source changes and release notes.

For local development, run `make check`; use `make release` to produce an optimized binary after checks pass.
