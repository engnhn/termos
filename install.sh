#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="engnhn/termos"
ASSET="termos-linux-x86_64"
SIGNER_WORKFLOW="engnhn/termos/.github/workflows/release.yml"

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
    echo "Verified release installer currently supports Linux x86_64 only." >&2
    exit 1
fi
for cmd in gh install; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Required command '$cmd' is missing. Install GitHub CLI and coreutils, then retry." >&2
        exit 1
    fi
done
gh_version="$(gh version | awk 'NR == 1 { print $3 }')"
if [[ ! "$gh_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || [[ "$(printf '%s\n' 2.102.0 "$gh_version" | sort -V | head -n1)" != "2.102.0" ]]; then
    echo "GitHub CLI 2.102.0 or newer is required for safe attestation verification." >&2
    exit 1
fi

tmp_dir="$(mktemp -d)"
chmod 700 "$tmp_dir"
trap 'rm -rf "$tmp_dir"' EXIT

tag="$(gh release view --repo "$REPOSITORY" --json tagName --jq .tagName)"
if [[ ! "$tag" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "GitHub returned an invalid release tag." >&2
    exit 1
fi

gh release download "$tag" --repo "$REPOSITORY" --pattern "$ASSET" --dir "$tmp_dir"
gh attestation verify "$tmp_dir/$ASSET" --repo "$REPOSITORY" --signer-workflow "$SIGNER_WORKFLOW"

install_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
install -d -m 700 "$install_dir"
install -m 755 "$tmp_dir/$ASSET" "$install_dir/termos"
echo "Installed verified $tag to $install_dir/termos"
case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "Add $install_dir to PATH to run termos." ;;
esac
