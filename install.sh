#!/usr/bin/env bash
# install.sh — build & install the rust-my-claude binary, then run its
# interactive theme picker. All settings.json / config handling lives in the
# `rust-my-claude init` CLI; this script only bootstraps the binary.
# Usage:  bash install.sh
#         bash install.sh --dir ~/.local/bin   # custom install dir
#         bash install.sh --no-init            # install binary only, skip picker
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="rust-my-claude"
RUN_INIT=1

# ── parse args ────────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--dir) INSTALL_DIR="$2"; shift 2 ;;
    --no-init) RUN_INIT=0; shift ;;
    -h|--help)
      echo "Usage: bash install.sh [--dir DIR] [--no-init]"
      echo "  --dir DIR   Install binary to DIR (default: ~/.local/bin)"
      echo "  --no-init   Install the binary only; skip the theme picker"
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
command -v cargo &>/dev/null || die "cargo not found. Install Rust from https://rustup.rs/ then re-run."
ok "Rust $(rustc --version | awk '{print $2}')"

# ── build ─────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo ""
echo "Building release binary…"
# Pin the target dir so the copy below works regardless of any global
# CARGO_TARGET_DIR / ~/.cargo/config.toml target-dir override.
cargo build --release --quiet --manifest-path "${SCRIPT_DIR}/Cargo.toml" --target-dir "${SCRIPT_DIR}/target"
ok "Build complete"

# ── install binary ────────────────────────────────────────────────────────────
mkdir -p "${INSTALL_DIR}"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
install -m 755 "${SCRIPT_DIR}/target/release/${BINARY_NAME}" "${BINARY_PATH}"
ok "Binary installed to ${BINARY_PATH}"

# ── check PATH ────────────────────────────────────────────────────────────────
if ! echo "$PATH" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
  warn "${INSTALL_DIR} is not in your PATH."
  echo "   Add this to your shell profile (~/.bashrc / ~/.zshrc):"
  echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

# ── hand off to the theme picker ──────────────────────────────────────────────
echo ""
if [[ "${RUN_INIT}" -eq 1 ]]; then
  echo "Launching theme picker…"
  echo ""
  "${BINARY_PATH}" init
else
  ok "Binary installed. Run '${BINARY_NAME} init' to pick a theme and wire up Claude Code."
fi
