#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
APP_DIR="${ROOT_DIR}/firmware-test-rp2040"
DIST_DIR="${ROOT_DIR}/dist"
TARGET="thumbv6m-none-eabi"
ELF_PATH="${APP_DIR}/target/${TARGET}/release/katasam-rp2040-test-fw"
UF2_BASE="${DIST_DIR}/katasam-rp2040-test-fw"

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

variants=(
  "boot2-w25q080:w25q080"
  "boot2-is25lp080:is25lp080"
  "boot2-generic-03h:generic03h"
)

for entry in "${variants[@]}"; do
  feature="${entry%%:*}"
  suffix="${entry##*:}"
  out_path="${UF2_BASE}-${suffix}.uf2"

  cargo build --release --target "${TARGET}" --no-default-features --features "${feature}"

  if [[ ! -f "${ELF_PATH}" ]]; then
    echo "expected ELF not found: ${ELF_PATH}" >&2
    exit 1
  fi

  elf2uf2-rs "${ELF_PATH}" "${out_path}"
  echo "UF2 generated: ${out_path}"
done

cp -f "${UF2_BASE}-w25q080.uf2" "${UF2_BASE}.uf2"
echo "Default UF2 symlink copy updated: ${UF2_BASE}.uf2 -> ${UF2_BASE}-w25q080.uf2"

rescue_path="${DIST_DIR}/katasam-rp2040-rescue-w25q080.uf2"
cargo build --release --target "${TARGET}" --no-default-features --features "boot2-w25q080 rescue-mode"

if [[ ! -f "${ELF_PATH}" ]]; then
  echo "expected ELF not found: ${ELF_PATH}" >&2
  exit 1
fi

elf2uf2-rs "${ELF_PATH}" "${rescue_path}"
echo "UF2 generated: ${rescue_path}"
