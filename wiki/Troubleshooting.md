# Troubleshooting

## `termos: command not found`

The installer uses `~/.local/bin` by default. Add it to `PATH`, then open a new shell or reload your shell profile:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Installer or updater cannot verify a release

Check `gh version`; Termos requires GitHub CLI 2.102.0 or newer for artifact attestation verification. Confirm the machine is Linux x86_64 and that the latest release contains `termos-linux-x86_64`. Releases without a matching artifact attestation are rejected.

## Password authentication does not work

Check that the desktop Secret Service is running and unlocked. GNOME Keyring and KWallet are common providers. Termos does not fall back to writing the password into the connection JSON. You can use an SSH key instead; SSH handles key passphrases.

## SSH cannot connect

Try the same host from a terminal with the system `ssh` client. Check the hostname, port, username, network access, key path, and host-key prompt. Termos uses the system SSH client and respects its host-key checks and SSH configuration.

## MCP server is missing in Antigravity

Check that `command` points to the installed binary using an absolute path, and that `args` contains `"mcp"`. Verify the JSON in `~/.gemini/config/mcp_config.json`, then restart Antigravity and inspect **Additional Options → MCP Servers**. Launch Termos manually with `termos mcp` to see startup errors in the terminal.

## MCP command times out or returns partial output

Command timeouts default to 30 seconds and have a 120-second maximum. stdout and stderr are each capped at 32 KiB; longer output is truncated. A local timeout stops the SSH process group, but a remote process that detached itself may continue.

When asking for help, include the Termos version and the exact error. Remove hostnames, IP addresses, usernames, command output, and credentials from logs first.
