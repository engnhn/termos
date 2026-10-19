Output demo/termos-tour.mp4
Set Width 1920
Set Height 1080
Set FontSize 23
Set TypingSpeed 65ms
Set Framerate 30
Set Theme "Dracula"

# Start the actual TUI against an isolated sample profile.
Hide
Type "export XDG_CONFIG_HOME=__TERMO_DEMO_CONFIG__"
Enter
Type "./target/release/termos"
Enter
Sleep 2s
Show
Sleep 2s

# Add a sample host with the built-in form.
Type "a"
Sleep 1s
Type "cache-01"
Tab
Type "192.0.2.40"
Tab
Tab
Tab
Tab
Tab
Type "staging"
Sleep 1s
Ctrl+S
Sleep 2s

# Search, then clear the query and browse group filters.
Type "/"
Sleep 500ms
Type "stage"
Sleep 2s
Escape
Sleep 500ms
Escape
Sleep 700ms
Type "g"
Sleep 500ms
Down
Down
Enter
Sleep 2s
Type "g"
Sleep 500ms
Enter
Sleep 1500ms

# Inspect, but do not execute, a saved remote command.
Type "c"
Sleep 2500ms
Down
Sleep 1500ms
Escape
Sleep 500ms

# Show the server editing options.
Type "e"
Sleep 2s
Escape
Sleep 2s

# Show the delete confirmation and decline it.
Type "d"
Sleep 1500ms
Type "n"
Sleep 1s
Type "q"
Hide
Sleep 500ms
Type "clear"
Enter
Show
Sleep 500ms

# CLI: save and inspect a quick command; this only updates the temporary fixture.
Type "./target/release/termos qc add web-01 --name 'memory' --cmd 'free -h'"
Enter
Sleep 1s
Type "./target/release/termos qc list web-01"
Enter
Sleep 2s
Escape
Sleep 1s

# CLI and MCP entry points; no SSH connection is opened in this demo.
Type "./target/release/termos --help"
Enter
Sleep 2s
Type "./target/release/termos mcp --help"
Enter
Sleep 3s
Type "python3 demo/list_mcp_tools.py ./target/release/termos"
Enter
Sleep 4s
