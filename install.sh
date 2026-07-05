#!/usr/bin/env bash

# Stop on any error
set -e

echo "=== Termos CLI Installer ==="
echo "Building the project in release mode..."
cargo build --release

echo "Installing binary to /usr/local/bin/termos..."
# Copy the binary. Using sudo as /usr/local/bin is normally root-owned.
sudo cp target/release/termos /usr/local/bin/termos
sudo chmod +x /usr/local/bin/termos

echo "Installing shell autocompletions..."

# Bash completion
if [ -d /usr/share/bash-completion/completions ]; then
    sudo cp completions/termos.bash /usr/share/bash-completion/completions/termos
    echo "✔ Installed Bash autocompletion to /usr/share/bash-completion/completions/termos"
elif [ -d /etc/bash_completion.d ]; then
    sudo cp completions/termos.bash /etc/bash_completion.d/termos
    echo "✔ Installed Bash autocompletion to /etc/bash_completion.d/termos"
else
    echo "⚠ Bash completion folder not found. Skipping Bash completions."
fi

# Zsh completion
if [ -d /usr/local/share/zsh/site-functions ]; then
    sudo cp completions/termos.zsh /usr/local/share/zsh/site-functions/_termos
    echo "✔ Installed Zsh autocompletion to /usr/local/share/zsh/site-functions/_termos"
else
    echo "⚠ Zsh site-functions folder not found. Skipping Zsh completions."
fi

echo "=========================================="
echo "✔ Installation successful!"
echo "You can now run 'termos' from anywhere."
echo "Note: Restart your terminal or source your shell configuration to load autocompletions."
echo "=========================================="
