#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

mkdir -p "$tmp_dir/config/termos" "$tmp_dir/chromium-config"
cp "$repo_root/demo/fixture/termos/connections.json" "$tmp_dir/config/termos/connections.json"
sed "s|__TERMO_DEMO_CONFIG__|$tmp_dir/config|g" \
    "$repo_root/demo/termos.tape" > "$tmp_dir/termos.tape"
export XDG_CONFIG_HOME="$tmp_dir/chromium-config"

cd "$repo_root"
vhs "$tmp_dir/termos.tape"
