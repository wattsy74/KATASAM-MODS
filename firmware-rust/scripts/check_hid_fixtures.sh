#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HID_DIR="${ROOT_DIR}/fixtures/hid"

required=(
  "idle_YYYYMMDD.txt"
  "single_actions_YYYYMMDD.txt"
  "combo_actions_YYYYMMDD.txt"
)

status=0

echo "Checking HID fixture placeholders"
for file_name in "${required[@]}"; do
  path="${HID_DIR}/${file_name}"
  if [[ ! -f "${path}" ]]; then
    echo "missing: ${path}" >&2
    status=1
    continue
  fi

  # Ignore comment and blank lines when deciding if a file has capture data.
  data_lines="$(awk 'BEGIN{n=0} { if ($0 !~ /^\s*($|#)/) n++ } END{ print n }' "${path}")"
  if [[ "${data_lines}" -eq 0 ]]; then
    echo "empty capture: ${path} (only comments/blank lines)" >&2
    status=1
  else
    echo "ok: ${path} (${data_lines} data lines)"
  fi
done

if [[ ${status} -eq 0 ]]; then
  echo "HID fixture check passed."
else
  echo "HID fixture check failed." >&2
fi

exit ${status}
