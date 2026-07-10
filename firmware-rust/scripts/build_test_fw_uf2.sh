#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
APP_DIR="${ROOT_DIR}/firmware-test-rp2040"
DIST_DIR="${ROOT_DIR}/dist"
TARGET="thumbv6m-none-eabi"
ELF_PATH="${APP_DIR}/target/${TARGET}/release/katasam-rp2040-test-fw"
UF2_PATH="${DIST_DIR}/katasam-rp2040-test-fw.uf2"

mkdir -p "${DIST_DIR}"

if ! command -v cargo >/dev/null 2>&1; then
  if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
  fi
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust toolchain first." >&2
  exit 1
fi

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  rustup target add "${TARGET}"
fi

if ! command -v elf2uf2-rs >/dev/null 2>&1; then
  cargo install elf2uf2-rs
fi

cd "${APP_DIR}"
cargo build --release --target "${TARGET}"

if [[ ! -f "${ELF_PATH}" ]]; then
  echo "expected ELF not found: ${ELF_PATH}" >&2
  exit 1
fi

elf2uf2-rs "${ELF_PATH}" "${UF2_PATH}"

echo "UF2 generated: ${UF2_PATH}"
