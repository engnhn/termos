#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

mkdir -p "$tmp_dir/config/termos" "$tmp_dir/chromium-config"
cp "$repo_root/demo/fixture/termos/connections.json" "$tmp_dir/config/termos/connections.json"
sed "s|__TERMO_DEMO_CONFIG__|$tmp_dir/config|g" \
    "$repo_root/demo/termos-tour.tape" > "$tmp_dir/termos-tour.tape"

export XDG_CONFIG_HOME="$tmp_dir/chromium-config"
cd "$repo_root"
vhs "$tmp_dir/termos-tour.tape"

cp "$repo_root/demo/termos-tour.mp4" "$tmp_dir/termos-tour-raw.mp4"
ffmpeg -y -i "$tmp_dir/termos-tour-raw.mp4" \
    -vf "subtitles=filename=$repo_root/demo/termos-tour.srt:force_style='FontName=DejaVu Sans Mono,FontSize=14,PrimaryColour=&H00FFFFFF,Outline=1,Shadow=0,Alignment=2,MarginV=26,BorderStyle=3,BackColour=&H80000000'" \
    -c:v libx264 -preset fast -crf 18 -pix_fmt yuv420p -an \
    "$tmp_dir/termos-tour-captioned.mp4"
mv "$tmp_dir/termos-tour-captioned.mp4" "$repo_root/demo/termos-tour.mp4"

ffmpeg -y -i "$repo_root/demo/termos-tour.mp4" -t 18 \
    -filter_complex "fps=12,scale=1440:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128:stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" \
    -loop 0 "$repo_root/demo/termos-demo.gif"
