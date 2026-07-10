# Phase 0 Capture Guide

Purpose: capture baseline behavior from current CircuitPython firmware before Rust parity work.

## Outputs to Capture

- Serial protocol trace (command/response)
- HID input report bytes under known button/axis actions
- Config file round-trip behavior (`READFILE`, `WRITEFILE`)

Store artifacts under:
- `firmware-rust/fixtures/protocol/`
- `firmware-rust/fixtures/hid/`

## Preconditions

1. Current known-good CircuitPython firmware on device
2. KATASAM Configurator available and able to connect
3. Device connected over USB data port

## Serial Trace Procedure

1. Start a raw serial log session.
2. Send each command listed in `fixtures/protocol/command-matrix.md`.
3. Record full responses including ACK lines and END delimiters.
4. Save one file per command family and one full-session trace.

Recommended filenames:
- `protocol/session_full_YYYYMMDD.txt`
- `protocol/readfile_config_YYYYMMDD.txt`
- `protocol/writefile_roundtrip_YYYYMMDD.txt`

## HID Capture Procedure

1. Capture idle reports for 10 seconds.
2. Capture single-action reports:
   - each fret press/release
   - strum up/down
   - guide/start/select
   - tilt transitions
   - whammy min/mid/max
3. Capture combined-action reports:
   - fret + strum combinations
   - dpad diagonals and hat transitions
4. Save as timestamped hex logs.

Recommended filenames:
- `hid/idle_YYYYMMDD.txt`
- `hid/single_actions_YYYYMMDD.txt`
- `hid/combo_actions_YYYYMMDD.txt`

## Validation Notes

- Keep captures on the same firmware build for consistency.
- Note host OS and app version used for capture.
- If response timing appears unstable, repeat and keep both traces.

## Exit Criteria For Phase 0

- Golden serial traces checked in for core command families
- Golden HID traces checked in for required action matrix
- Any ambiguities documented in `fixtures/KNOWN_GAPS.md`

## Replay Integration

After creating a real capture fixture pair, replay it against the Rust bridge:

```bash
cd firmware-rust
./scripts/replay_real_capture_fixture.sh capture-001
```

Use `fixtures/protocol/REAL_CAPTURE_WORKFLOW.md` for fixture naming conventions.
