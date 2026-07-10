#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FIXTURES_DIR="${ROOT_DIR}/fixtures/protocol"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found in PATH" >&2
  exit 1
fi

# Ensure we run from workspace root for stable relative paths.
cd "${ROOT_DIR}"

declare -a CASES=(
  "basic"
  "control"
  "failure"
)

overall_status=0

echo "Running protocol fixture replay against --protocol-stdio bridge"
for case_name in "${CASES[@]}"; do
  cmd_file="${FIXTURES_DIR}/cmd-${case_name}-session.txt"
  golden_file="${FIXTURES_DIR}/golden-${case_name}-session.txt"
  output_file="${FIXTURES_DIR}/out-${case_name}-session.txt"

  if [[ ! -f "${cmd_file}" ]]; then
    echo "[${case_name}] missing command file: ${cmd_file}" >&2
    overall_status=1
    continue
  fi

  if [[ ! -f "${golden_file}" ]]; then
    echo "[${case_name}] missing golden file: ${golden_file}" >&2
    overall_status=1
    continue
  fi

  echo "[${case_name}] replaying commands"
  cat "${cmd_file}" | cargo run -q -p katasam-firmware-main -- --protocol-stdio > "${output_file}"

  if diff -u "${golden_file}" "${output_file}" > "${FIXTURES_DIR}/diff-${case_name}-session.txt"; then
    rm -f "${FIXTURES_DIR}/diff-${case_name}-session.txt"
    echo "[${case_name}] OK"
  else
    echo "[${case_name}] MISMATCH (see ${FIXTURES_DIR}/diff-${case_name}-session.txt)" >&2
    overall_status=1
  fi
done

if [[ ${overall_status} -eq 0 ]]; then
  echo "All protocol fixture replays matched golden outputs."
else
  echo "One or more fixture replays mismatched golden outputs." >&2
fi

exit ${overall_status}
