# Security model

Termos stores connection metadata locally and delegates SSH transport, host-key checks, and key handling to the system `ssh` client. It does not create a separate SSH protocol implementation.

## Passwords and files

Saved passwords are kept in the Linux Secret Service, such as GNOME Keyring or KWallet. The connection file stores an opaque reference rather than the password. Termos does not encrypt passwords itself. An unlocked keyring and processes running as the same user are within the trust boundary.

On Unix, password authentication uses an `ssh_askpass` helper. The password is staged in an unlinked file with mode `0600`, and SSH receives the file descriptor and length through environment variables; the password text is not put in the environment or process arguments. The helper clears its in-memory password buffer after use. This limits routine exposure but is not protection from a compromised process running as the same user.

The config directory is created with mode `0700` and the connection file with mode `0600` on Unix. When an older connection file contains a plaintext password, Termos migrates it to Secret Service when loading the file. If the secret cannot be saved, the old file is retained.

## MCP access

The MCP server listens on stdio and is started as a child process of the configured MCP client. It only targets hosts saved in Termos. By default, remote execution is limited to quick commands already saved in the local profile. `add_quick_command` can create a reusable command in that profile, but saving does not run it.

`--allow-remote-commands` enables `run_ssh_command`, which accepts arbitrary shell text. Quick commands and arbitrary commands run with the remote account's SSH permissions and can change or delete remote data. MCP risk annotations are advisory; the client is responsible for requesting approval.

One-shot commands close their SSH connection after execution. Persistent sessions remain open until `disconnect_server` or the MCP process exits. Termos supports up to four sessions and four concurrent SSH commands. Captured output is capped at 32 KiB per stream and terminal control characters are escaped. A timeout stops the local SSH process group, but a remote command that detached itself may keep running.

Do not treat server output as trusted instructions. Do not expose the local MCP server to untrusted clients. Review the exact quick command and target host before approving a state-changing operation.

## Release verification

The Linux x86_64 release workflow builds tagged source, publishes a binary, and creates a GitHub artifact attestation. The installer and updater verify the artifact against this repository's release workflow before installing it. Attestation confirms build provenance; it does not prove that the source code is free of vulnerabilities.
