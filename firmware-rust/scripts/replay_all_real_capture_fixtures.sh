#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FIXTURES_DIR="${ROOT_DIR}/fixtures/protocol"

cd "${ROOT_DIR}"

overall_status=0

for cmd_path in "${FIXTURES_DIR}"/cmd-real-*-session.txt; do
  [[ -e "${cmd_path}" ]] || continue

  file_name="$(basename "${cmd_path}")"
  fixture_id="${file_name#cmd-real-}"
  fixture_id="${fixture_id%-session.txt}"

  echo "[${fixture_id}] replaying"
  if ./scripts/replay_real_capture_fixture.sh "${fixture_id}"; then
    echo "[${fixture_id}] OK"
  else
    echo "[${fixture_id}] MISMATCH" >&2
    overall_status=1
  fi

done

if [[ ${overall_status} -eq 0 ]]; then
  echo "All real-capture fixtures matched golden outputs."
else
  echo "One or more real-capture fixtures mismatched golden outputs." >&2
fi

exit ${overall_status}
