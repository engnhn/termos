# Demo recording

`termos-demo.gif` records the real Termos interface with sample connection profiles from `config/termos/connections.json`. The addresses are from the documentation-only `192.0.2.0/24` range. The tape searches, filters groups, and opens the quick-command list; it does not connect or run a remote command.

To recreate the GIF from the repository root, build Termos and install VHS, `ttyd`, and FFmpeg, then run:

```bash
make release
XDG_CONFIG_HOME="$PWD/demo/config" vhs demo/termos.tape
```

In restricted environments where Chromium cannot use its sandbox, set `VHS_NO_SANDBOX=1` for the recording process.
