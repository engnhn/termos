#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
RESET='\033[0m'
BOLD='\033[1m'

echo -e "${CYAN}${BOLD}⚡ Termos Connection Manager Installation${RESET}"
echo -e "${CYAN}====================================================${RESET}"

for cmd in git cargo; do
    if ! command -v $cmd &> /dev/null; then
        echo -e "${RED}[✗] Error: $cmd is not installed.${RESET}"
        if [ "$cmd" = "cargo" ]; then
            echo -e "${YELLOW}Please install the Rust toolchain by running: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${RESET}"
        fi
        exit 1
    fi
done

TEMP_DIR=""
WORK_DIR=$(pwd)

if [ ! -f "Cargo.toml" ]; then
    echo -e "${YELLOW}[⚡] Remote run detected. Cloning repository...${RESET}"
    TEMP_DIR=$(mktemp -d -t termos-XXXXXXXXXX)
    git clone --depth 1 https://github.com/engnhn/termos.git "$TEMP_DIR" &>/dev/null
    WORK_DIR="$TEMP_DIR"
fi

cd "$WORK_DIR"

echo -e "${GREEN}[+] Building optimized release binary...${RESET}"
cargo build --release

echo -e "${GREEN}[+] Installing binary to /usr/local/bin/termos...${RESET}"
sudo cp target/release/termos /usr/local/bin/termos
sudo chmod +x /usr/local/bin/termos

echo -e "${GREEN}[+] Configuring shell completions...${RESET}"

if [ -d /usr/share/bash-completion/completions ]; then
    sudo cp completions/termos.bash /usr/share/bash-completion/completions/termos
    echo -e "${GREEN}    [✓] Bash completions installed to /usr/share/bash-completion/completions/termos${RESET}"
elif [ -d /etc/bash_completion.d ]; then
    sudo cp completions/termos.bash /etc/bash_completion.d/termos
    echo -e "${GREEN}    [✓] Bash completions installed to /etc/bash_completion.d/termos${RESET}"
else
    echo -e "${YELLOW}    [!] Bash completions folder not found. Skipping.${RESET}"
fi

if [ -d /usr/local/share/zsh/site-functions ]; then
    sudo cp completions/termos.zsh /usr/local/share/zsh/site-functions/_termos
    echo -e "${GREEN}    [✓] Zsh completions installed to /usr/local/share/zsh/site-functions/_termos${RESET}"
else
    echo -e "${YELLOW}    [!] Zsh site-functions folder not found. Skipping.${RESET}"
fi

if [ -n "$TEMP_DIR" ]; then
    rm -rf "$TEMP_DIR"
fi

echo -e "${CYAN}====================================================${RESET}"
echo -e "${GREEN}${BOLD}✔ Installation Successful!${RESET}"
echo -e "You can now run ${BOLD}termos${RESET} from anywhere."
echo -e "${YELLOW}Note: Please restart your shell or run 'source ~/.bashrc' to load completions.${RESET}"
echo -e "${CYAN}====================================================${RESET}"
