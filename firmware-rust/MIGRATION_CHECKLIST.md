# Rust Migration Checklist

Date started: 2026-07-10
Branch: firmware-rebuild-native-xbox-ps-support

## Phase 0 - Capture Baseline

- [x] Prepare capture guide and artifact folders
- [ ] Capture HID descriptor/report bytes from current CircuitPython firmware
- [ ] Capture full configurator serial command/response traces
- [ ] Freeze golden trace files in repo

## Phase 1 - Rust Foundation

- [x] Rust workspace scaffolded
- [x] Rust toolchain installed locally
- [x] RP2040 target added (thumbv6m-none-eabi)
- [ ] Board bring-up binary flashes successfully

## Phase 2 - Configurator Parity

- [x] Protocol parser crate scaffolded
- [x] Expand parser to cover legacy command surface
- [x] Add protocol engine state machine for write/merge session flow
- [x] Add response envelope utilities (ACK, START_/END_, WRITEFILE:READY, STREAM:READY)
- [x] Add host-side command execution harness (READFILE/WRITEFILE/IMPORTUSER)
- [x] Add fixture-based golden protocol session test for output stability
- [x] Validate streaming WRITEFILE completion semantics in runtime harness
- [x] Add golden protocol session for control/pin/LED command family
- [x] Add golden protocol session for failure/error-path command handling
- [x] Add app-level line-based protocol transport adapter (stdio bridge)
- [x] Add automated fixture replay script for parity checks against stdio bridge
- [x] Prepare first real-capture fixture placeholders and intake workflow
- [x] Capture and pass real-capture replay gate for `capture-001`
- [x] Capture and pass real-capture replay gate for `capture-002`
- [ ] Implement full command coverage from current serial handler
- [ ] Implement safe config read/write with atomic persistence
- [ ] Verify configurator core flows pass

## Phase 3 - PC Parity

- [x] PC profile scaffolded
- [ ] Match existing report behavior in live tests
- [ ] Validate Clone Hero, YARG, FNF behavior

## Phase 4 - Console Profiles

- [ ] Xbox One GH3 experimental profile
- [ ] Xbox Series X|S RB4 experimental profile
- [ ] PS4 RB4 profile
- [ ] Xbox 360 GH3 profile
- [ ] PS3 GH3 profile

## Non-Negotiable Safety Gates

- [ ] Configurator read/write works
- [ ] Preset and calibration flows work
- [ ] Reboot and reconnect flows work
- [ ] Existing config schema backward compatibility verified
