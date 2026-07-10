# Native Console Support Plan (Using Santroller as Reference)

Date: 2026-07-10
Branch: firmware-rebuild-native-xbox-ps-support

## Goal

Implement native platform support profiles for:
- Xbox 360 + Guitar Hero 3
- Xbox One + Guitar Hero 3
- Xbox Series X|S + Rock Band 4
- PS3 + Guitar Hero 3
- PS4 + Rock Band 4
- Preserve PC compatibility (Clone Hero, YARG, FNF, etc.)

## Important Constraint

Full compatibility with KATASAM Guitars Configurator must remain intact.

## Firmware Language Decision

Primary implementation language for console-native support is Rust.

Rationale:
- Deterministic performance and tighter control for timing-sensitive HID/report behavior
- Better long-term maintainability for multi-profile platform support
- Strong type safety for protocol and descriptor handling

CircuitPython remains only as a legacy baseline reference during migration.

## Reference Strategy

Use Santroller as a behavioral reference source, not a copy source.
- Capture device identity behavior (VID/PID/interface layout)
- Capture HID report descriptor characteristics
- Capture report cadence/timing and button axis mappings
- Capture platform-specific quirks per game

Implementation rule:
- Do not paste source code from external projects into this repo.
- Re-implement behavior in KATASAM firmware architecture.

## Why Start With Xbox One / Series

This is the highest-risk target due to stricter modern console expectations and game-specific instrument behavior.

## Firmware Architecture Direction

Split firmware into a stable core plus profile adapters:
- Core: input scanning, debouncing, mappings, calibration state
- Profile adapters: platform identity, descriptors, report shaping, timing
- Transport/config layer: existing serial/config interfaces used by configurator

This prevents console work from breaking configurator support.

## Compatibility Matrix (Initial)

| Platform | Game | Priority | Risk | Hardware Available | Status |
|---|---|---:|---:|---|---|
| Xbox One | Guitar Hero 3 | 1 | High | Maybe | Planned |
| Xbox Series X|S | Rock Band 4 | 1 | High | Not yet | Planned |
| Xbox 360 | Guitar Hero 3 | 2 | Medium | Unknown | Planned |
| PS4 | Rock Band 4 | 2 | Medium | Console yes, game not yet | Planned |
| PS3 | Guitar Hero 3 | 3 | Low-Med | Yes | Planned |
| PC | Clone Hero / YARG / FNF | 0 | Low | Yes | Must not regress |

## Deliverables

1. Profile abstraction in firmware codebase
2. Per-platform descriptor/report profile definitions
3. Regression suite for PC behavior and configurator protocol
4. Per-platform validation checklist and known-issues log
5. Stage Tour readiness checklist with profile-level override hooks
6. Rust firmware migration plan with parity checkpoints

## Immediate Next Tasks

1. Baseline current KATASAM descriptor/report behavior on PC
2. Record configurator protocol contract (serial commands, config schema)
3. Build Rust HAL + USB/HID scaffolding for RP2040 target
4. Implement a PC parity profile in Rust before console profiles
5. Add Xbox One/X|S experimental profile behind compile/runtime flag
6. Test no-regression path in configurator after each firmware change

## Configurator Safety Gates

Any firmware change is blocked from merge unless all pass:
- Config read/write still works
- Preset switching still works
- Calibration flows still work
- Device reconnect and reboot behavior still works
- Existing config schema remains backward compatible

## Stage Tour Readiness Principle

If a device behaves as a native, stable target on current titles now, future title support should require profile tweaks rather than core rewrites.
