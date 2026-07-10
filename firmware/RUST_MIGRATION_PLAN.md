# Rust Firmware Migration Plan

Date: 2026-07-10
Branch: firmware-rebuild-native-xbox-ps-support

## Objective

Migrate firmware implementation from CircuitPython to Rust to support native console compatibility targets while preserving full configurator compatibility and PC support.

## Current Baseline (CircuitPython Responsibilities)

- Main loop and orchestration: code.py
- HID gamepad report generation and sending: gamepad.py
- Serial command protocol and file/config operations: serial_handler.py
- Hardware pin setup and peripherals: hardware.py
- Config parsing and normalization: utils.py

## Rust Target Architecture

- crates/firmware-core
  - Input state model, mappings, calibration, preset application
- crates/platform-profiles
  - xbox360, xboxone, xboxseries, ps3, ps4, pc profile adapters
  - USB identity, descriptors, report packing, timing policies
- crates/config-protocol
  - Serial command parser/encoder
  - Config schema compatibility layer for existing configurator commands
- crates/board-rp2040
  - RP2040 hardware abstraction, GPIO/ADC/timers, watchdog, reboot paths
- app/main
  - Main loop wiring, scheduler, profile selection, diagnostics hooks

## Hard Compatibility Contracts

1. Existing configurator serial commands must continue to work.
2. Existing config schema keys and semantics must stay backward compatible.
3. PC behavior for Clone Hero, YARG, FNF must not regress.
4. Reboot and BOOTSEL update workflows must remain reliable.

## Migration Phases

## Phase 0 - Protocol and Behavior Capture

- Capture current HID descriptors and report bytes from baseline firmware
- Capture serial command request/response behavior and timing
- Freeze a compatibility test corpus for regression checks

Exit criteria:
- Golden files for HID reports and serial protocol traces are checked in.

## Phase 1 - Rust Skeleton and Tooling

- Set up Rust workspace and RP2040 target toolchain
- Add minimal board bring-up and heartbeat loop
- Add CI build checks for firmware crate(s)

Exit criteria:
- Rust firmware builds reproducibly and flashes on hardware.

## Phase 2 - Configurator Protocol Parity

- Re-implement serial commands used by configurator
- Re-implement config file read/write operations with safe persistence
- Validate ACK/response behavior matches expected patterns

Exit criteria:
- Configurator can connect and complete core flows without code changes.

## Phase 3 - PC Profile Parity

- Implement pc profile HID descriptor/reporting in Rust
- Match baseline behavior for fret/strum/whammy/hat mappings
- Validate no regressions in Clone Hero, YARG, and FNF

Exit criteria:
- PC parity profile passes regression tests and manual play testing.

## Phase 4 - Console Profiles

- Implement xboxone/xboxseries profiles first (highest risk)
- Implement ps4 profile and validate Rock Band 4 behavior
- Implement xbox360 and ps3 profiles
- Keep profile selection isolated from core logic

Exit criteria:
- Platform/game matrix reaches initial playable state on target hardware.

## Phase 5 - Stage Tour Readiness Layer

- Add per-title quirks layer for profile overrides
- Keep defaults stable while enabling targeted compatibility fixes

Exit criteria:
- Stage Tour deltas can be implemented as profile-level changes.

## Test Matrix (Minimum)

- Configurator: connect, read config, write config, calibration, presets
- PC: Clone Hero, YARG, FNF input and timing validation
- Xbox: 360 GH3, One GH3, Series X|S RB4
- PlayStation: PS3 GH3, PS4 RB4

## Risk Register

- Modern Xbox authentication and strict identity checks
- Descriptor/report timing subtlety affecting note detection
- Accidental breakage of configurator protocol behavior

## Immediate Next Actions

1. Stand up a Rust workspace under firmware-rust and board target config.
2. Implement serial protocol parser with baseline ACK behavior.
3. Add PC profile first, then branch to console profiles.
