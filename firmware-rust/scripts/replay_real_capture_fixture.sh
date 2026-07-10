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

if ! command -v cargo >/dev/null 2>&1; then
  if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
  fi
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust or source ~/.cargo/env first." >&2
  exit 1
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/katasam-replay.XXXXXX")"
trap 'rm -rf "${tmp_dir}"' EXIT

extract_payload() {
  local ack_line="$1"
  local end_marker="$2"
  local output_path="$3"

  awk -v ack_line="$ack_line" -v end_marker="$end_marker" '
    $0 == ack_line { mode = 1; next }
    mode == 1 && index($0, "START_") == 1 { mode = 2; next }
    mode == 2 {
      line = $0
      if (line == end_marker) {
        exit
      }
      if (length(line) >= length(end_marker) && substr(line, length(line) - length(end_marker) + 1) == end_marker) {
        line = substr(line, 1, length(line) - length(end_marker))
        printf "%s", line
        exit
      }
      if (wrote_any) {
        printf "\n"
      }
      printf "%s", line
      wrote_any = 1
    }
  ' "${golden_file}" > "${output_path}"
}

config_override_file="${tmp_dir}/config.json"
user_presets_override_file="${tmp_dir}/user_presets.json"

runtime_version=""
device_name=""
device_uid=""
whammy_value=""
joystick_x=""
joystick_y=""
pin_reads=""

if [[ -f "${golden_file}" ]]; then
  runtime_version="$(awk '/^VERSION:/{sub(/^VERSION:/,""); print; exit}' "${golden_file}")"
  device_name="$(awk '/^ACK: READDEVICENAME$/{getline; print; exit}' "${golden_file}")"
  device_uid="$(awk '/^ACK: READUID$/{getline; print; exit}' "${golden_file}")"
  whammy_value="$(awk '/^WHAMMY:/{sub(/^WHAMMY:/,""); print; exit}' "${golden_file}")"
  joystick_x="$(awk -F: '/^JOYSTICK:X:/{print $3; exit}' "${golden_file}")"
  joystick_y="$(awk -F: '/^JOYSTICK:X:/{print $5; exit}' "${golden_file}")"
  pin_reads="$(awk '
    /^ACK: READPIN:/ {pin=$0; sub(/^ACK: READPIN:/, "", pin); next}
    pin != "" && /^PIN:/ {
      val=$0
      sub(/^PIN:[^:]*:/, "", val)
      if (pairs != "") { pairs = pairs ";" }
      pairs = pairs pin "=" val
      pin = ""
    }
    END { print pairs }
  ' "${golden_file}")"

  extract_payload "ACK: READFILE:/config.jso" "END_config.json" "${config_override_file}"
  extract_payload "ACK: READFILE:/user_prese" "END_user_presets.json" "${user_presets_override_file}"

  if [[ -s "${user_presets_override_file}" ]]; then
    printf '\n' >> "${user_presets_override_file}"
  fi
fi

cat "${cmd_file}" | \
  KATASAM_RUNTIME_VERSION="${runtime_version}" \
  KATASAM_DEVICE_NAME="${device_name}" \
  KATASAM_DEVICE_UID="${device_uid}" \
  KATASAM_WHAMMY_VALUE="${whammy_value}" \
  KATASAM_JOYSTICK_X="${joystick_x}" \
  KATASAM_JOYSTICK_Y="${joystick_y}" \
  KATASAM_PIN_READS="${pin_reads}" \
  KATASAM_CONFIG_OVERRIDE_FILE="${config_override_file}" \
  KATASAM_USER_PRESETS_OVERRIDE_FILE="${user_presets_override_file}" \
  cargo run -q -p katasam-firmware-main -- --protocol-stdio > "${out_file}"

if diff -u "${golden_file}" "${out_file}" > "${diff_file}"; then
  rm -f "${diff_file}"
  echo "real capture fixture '${fixture_id}' matched golden output"
else
  echo "real capture fixture '${fixture_id}' mismatch (see ${diff_file})" >&2
  exit 1
fi
