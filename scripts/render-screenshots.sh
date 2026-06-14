#!/usr/bin/env bash
# Render a PNG preview of every bundled theme into docs/screenshots/.
#
# Pipeline:  theme preview (ANSI) -> freeze (SVG, embeds Hack Nerd Font)
#            -> round the frame corners -> resvg (@2x PNG).
#
# Requirements: cargo, freeze (charmbracelet), resvg, a Hack Nerd Font .ttf,
# and python3. Fast (no headless browser) — ~1-2 min for all themes.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

BIN=./target/release/rust-my-claude
OUT=docs/screenshots
BG="#15161c"          # uniform window background
PAD=24                # symmetric padding around the statusline
SIZE=15               # font size
ZOOM=2                # 2x for crisp @retina PNGs
RADIUS=14             # rounded corner radius (added to the bg rect)

FONT="${RMC_FONT:-}"
if [[ -z "$FONT" ]]; then
  for c in "$HOME/Library/Fonts/HackNerdFontMono-Regular.ttf" \
           "$HOME/.local/share/fonts/HackNerdFontMono-Regular.ttf" \
           "/usr/share/fonts/truetype/hack/HackNerdFontMono-Regular.ttf"; do
    [[ -f "$c" ]] && FONT="$c" && break
  done
fi
[[ -f "$FONT" ]] || { echo "No Hack Nerd Font .ttf found; set RMC_FONT=/path/to/font.ttf"; exit 1; }

command -v freeze >/dev/null || { echo "freeze not found (brew install charmbracelet/tap/freeze)"; exit 1; }
command -v resvg  >/dev/null || { echo "resvg not found (brew install resvg)"; exit 1; }

cargo build --release --quiet
mkdir -p "$OUT"

render_one() {
  local theme="$1"
  local svg png
  svg="$(mktemp -t "rmc-$theme").svg"
  png="$OUT/$theme.png"
  "$BIN" theme preview "$theme" > "${svg}.ansi"
  freeze "${svg}.ansi" --font.file "$FONT" --font.family "Hack Nerd Font Mono" \
    --background "$BG" --padding "$PAD" --border.radius 0 \
    --font.size "$SIZE" --line-height 1.35 -o "$svg" >/dev/null 2>&1
  # Round the outer window: add rx/ry to the first (background) <rect>.
  python3 - "$svg" "$RADIUS" <<'PY'
import re, sys
p, r = sys.argv[1], sys.argv[2]
s = open(p).read()
s = re.sub(r'(<rect\b)', rf'\1 rx="{r}px" ry="{r}px"', s, count=1)
open(p, 'w').write(s)
PY
  resvg --zoom "$ZOOM" --use-font-file "$FONT" "$svg" "$png" 2>/dev/null
  rm -f "$svg" "${svg}.ansi"
  echo "  $theme"
}
export -f render_one
export BIN OUT BG PAD SIZE ZOOM RADIUS FONT

themes="$("$BIN" theme list | sed '1d' | awk '{print $1}')"
echo "Rendering $(echo "$themes" | wc -l | tr -d ' ') themes -> $OUT"
echo "$themes" | xargs -P 8 -I{} bash -c 'render_one "$@"' _ {}
echo "Done."
