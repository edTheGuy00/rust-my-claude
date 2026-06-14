#!/usr/bin/env bash
# install.sh — install rust-my-claude, then run its interactive theme picker.
#
#   --bin       (default) download the prebuilt binary — no Rust toolchain needed
#   --compile   clone (if needed) and compile from source with cargo
#
# Quick install (no clone required):
#   curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/edTheGuy00/rust-my-claude/main/install.sh | bash -s -- --compile
set -euo pipefail

REPO="edTheGuy00/rust-my-claude"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="rust-my-claude"
MODE="bin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin)     MODE="bin"; shift ;;
    --compile) MODE="compile"; shift ;;
    -h|--help)
      echo "Usage: install.sh [--bin | --compile]"
      echo "  --bin       Download the prebuilt binary (default; no Rust needed)"
      echo "  --compile   Clone and compile from source (requires Rust/cargo)"
      exit 0 ;;
    *) echo "Unknown option: $1 (use --bin or --compile)"; exit 1 ;;
  esac
done

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✓${NC} $*"; }
warn() { echo -e "${YELLOW}⚠${NC} $*"; }
die()  { echo -e "${RED}✗${NC} $*"; exit 1; }

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   rust-my-claude installer  (${MODE})"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

mkdir -p "${INSTALL_DIR}"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# ── detect the release target triple (for --bin) ───────────────────────────────
detect_target() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "${os}-${arch}" in
    Linux-x86_64)            echo "x86_64-unknown-linux-gnu" ;;
    Linux-aarch64|Linux-arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin-x86_64)           echo "x86_64-apple-darwin" ;;
    Darwin-arm64)            echo "aarch64-apple-darwin" ;;
    *) die "Unsupported platform: ${os}-${arch}. Try: install.sh --compile" ;;
  esac
}

install_bin() {
  command -v curl &>/dev/null || die "curl is required for --bin."
  local target url
  target="$(detect_target)"
  url="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}-${target}"
  echo "Downloading prebuilt binary (${target})…"
  if ! curl -fSL --proto '=https' "${url}" -o "${BINARY_PATH}.tmp"; then
    rm -f "${BINARY_PATH}.tmp"
    die "Download failed (no release asset for ${target}?). Try: install.sh --compile"
  fi
  chmod +x "${BINARY_PATH}.tmp"
  mv "${BINARY_PATH}.tmp" "${BINARY_PATH}"
  ok "Installed prebuilt binary to ${BINARY_PATH}"
}

install_compile() {
  command -v cargo &>/dev/null || die "cargo not found. Install Rust from https://rustup.rs/ — or use --bin."
  ok "Rust $(rustc --version | awk '{print $2}')"
  local src script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-.}")" 2>/dev/null && pwd || echo '')"
  if [[ -n "${script_dir}" && -f "${script_dir}/Cargo.toml" ]]; then
    src="${script_dir}"                      # running from inside a clone
    echo "Building from local checkout: ${src}"
  else
    command -v git &>/dev/null || die "git is required to clone the source."
    src="$(mktemp -d)/rust-my-claude"
    echo "Cloning ${REPO}…"
    git clone --depth 1 "https://github.com/${REPO}.git" "${src}"
  fi
  echo "Compiling release binary…"
  # Pin target-dir so the copy works regardless of any global cargo target-dir.
  cargo build --release --quiet --manifest-path "${src}/Cargo.toml" --target-dir "${src}/target"
  install -m 755 "${src}/target/release/${BINARY_NAME}" "${BINARY_PATH}"
  ok "Compiled and installed to ${BINARY_PATH}"
}

case "${MODE}" in
  bin)     install_bin ;;
  compile) install_compile ;;
esac

# ── PATH check ─────────────────────────────────────────────────────────────────
if ! echo "$PATH" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
  warn "${INSTALL_DIR} is not in your PATH. Add to your shell profile:"
  echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

# ── hand off to the theme picker (writes config + patches settings.json) ──────
echo ""
echo "Launching theme picker…"
echo ""
"${BINARY_PATH}" init
