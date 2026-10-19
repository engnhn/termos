# Demo recording

`termos-demo.gif` records the real Termos interface with sample profiles from `fixture/termos/connections.json`. The addresses are from the documentation-only `192.0.2.0/24` range. The tape searches, filters groups, and opens the quick-command list; it does not connect or run a remote command. The recording script copies this fixture to a temporary directory, separate from the browser's temporary profile and your real Termos configuration.

To recreate the GIF, build Termos and install VHS, `ttyd`, and FFmpeg, then run:

```bash
make release
./demo/record.sh
```

In restricted environments where Chromium cannot use its sandbox, set `VHS_NO_SANDBOX=1` for the recording process.
