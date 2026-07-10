#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <fixture-id>" >&2
  echo "example: $0 capture-001" >&2
  exit 2
fi

fixture_id="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FIXTURES_DIR="${ROOT_DIR}/fixtures/protocol"

cmd_file="${FIXTURES_DIR}/cmd-real-${fixture_id}-session.txt"
golden_file="${FIXTURES_DIR}/golden-real-${fixture_id}-session.txt"
out_file="${FIXTURES_DIR}/out-real-${fixture_id}-session.txt"
diff_file="${FIXTURES_DIR}/diff-real-${fixture_id}-session.txt"

if [[ ! -f "${cmd_file}" ]]; then
  echo "missing command file: ${cmd_file}" >&2
  exit 1
fi

if [[ ! -f "${golden_file}" ]]; then
  echo "missing golden file: ${golden_file}" >&2
  exit 1
fi

cd "${ROOT_DIR}"

cat "${cmd_file}" | cargo run -q -p katasam-firmware-main -- --protocol-stdio > "${out_file}"

if diff -u "${golden_file}" "${out_file}" > "${diff_file}"; then
  rm -f "${diff_file}"
  echo "real capture fixture '${fixture_id}' matched golden output"
else
  echo "real capture fixture '${fixture_id}' mismatch (see ${diff_file})" >&2
  exit 1
fi
