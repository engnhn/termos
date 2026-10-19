# Demo recording

`termos-demo.gif` is a short TUI preview. [`termos-tour.mp4`](termos-tour.mp4) is the full 1080p tour, with captions. It covers profile setup, search and group filters, saved commands, deletion confirmation, CLI command management, and the default MCP tools over stdio.

The tour uses profiles from `fixture/termos/connections.json`; its addresses are in the documentation-only `192.0.2.0/24` range. It saves a sample profile and quick command only inside a temporary configuration directory. It does not open an SSH session or run a remote command. Browser and application configuration stay isolated from your real Termos configuration.

To recreate the video and GIF, build Termos and install VHS, `ttyd`, FFmpeg, and Python 3, then run:

```bash
make release
./demo/record-tour.sh
```

In restricted environments where Chromium cannot use its sandbox, set `VHS_NO_SANDBOX=1` for the recording process.
