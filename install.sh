#!/usr/bin/env bash
# install.sh — build & install rust-my-claude (pure Rust, no npm/node/jq needed)
# Usage:  bash install.sh
#         bash install.sh --dir ~/.local/bin   # custom install dir
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"
SETTINGS_FILE="${HOME}/.claude/settings.json"
BINARY_NAME="rust-my-claude"

# ── parse args ────────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--dir) INSTALL_DIR="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: bash install.sh [--dir DIR]"
      echo "  --dir DIR   Install binary to DIR (default: ~/.local/bin)"
      exit 0 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✓${NC} $*"; }
warn() { echo -e "${YELLOW}⚠${NC} $*"; }
die()  { echo -e "${RED}✗${NC} $*"; exit 1; }

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   rust-my-claude installer"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ── check Rust ────────────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
  die "cargo not found. Install Rust from https://rustup.rs/ then re-run."
fi
RUST_VERSION=$(rustc --version | awk '{print $2}')
ok "Rust ${RUST_VERSION}"

# ── build ─────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo ""
echo "Building release binary…"
cd "${SCRIPT_DIR}"
# Pin the target dir so the copy below works regardless of any global
# CARGO_TARGET_DIR / ~/.cargo/config.toml target-dir override.
cargo build --release --quiet --target-dir "${SCRIPT_DIR}/target"
ok "Build complete"

# ── install binary ────────────────────────────────────────────────────────────
mkdir -p "${INSTALL_DIR}"
cp "${SCRIPT_DIR}/target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
ok "Binary installed to ${INSTALL_DIR}/${BINARY_NAME}"

# ── check PATH ────────────────────────────────────────────────────────────────
if ! echo "$PATH" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
  warn "${INSTALL_DIR} is not in your PATH."
  echo "   Add this to your shell profile (~/.bashrc / ~/.zshrc):"
  echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

# ── update ~/.claude/settings.json ───────────────────────────────────────────
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
echo ""
echo "Updating Claude Code settings…"

mkdir -p "$(dirname "${SETTINGS_FILE}")"

if [[ -f "${SETTINGS_FILE}" ]]; then
  # Back up existing settings
  BACKUP="${SETTINGS_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
  cp "${SETTINGS_FILE}" "${BACKUP}"
  ok "Backed up settings → $(basename "${BACKUP}")"

  # Patch with python3 (universally available, no jq needed)
  python3 - "${SETTINGS_FILE}" "${BINARY_PATH}" << 'PY'
import json, sys
path, cmd = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("statusLine", {}).update({"type": "command", "command": cmd, "padding": 0})
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
    f.write("\n")
PY
  ok "settings.json updated"
else
  # Create minimal settings
  cat > "${SETTINGS_FILE}" << JSON
{
  "statusLine": {
    "type": "command",
    "command": "${BINARY_PATH}",
    "padding": 0
  }
}
JSON
  ok "settings.json created"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}   Done!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Binary : ${BINARY_PATH}"
echo "  Config : ${SETTINGS_FILE}"
echo ""
echo "  Restart Claude Code to see the status line."
echo "  To test manually:"
echo '  echo '"'"'{"model":{"display_name":"Sonnet"},"workspace":{"current_dir":"'"${HOME}"'"},"context_window":{"used_percentage":42}}'"'"" | ${BINARY_PATH}"
echo ""
