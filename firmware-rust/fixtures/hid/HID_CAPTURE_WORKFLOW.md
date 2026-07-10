# HID Capture Workflow

## Goal

Capture and preserve real HID report baselines so Rust profile behavior can be validated against current firmware behavior.

## Required Fixture Files

- `fixtures/hid/idle_YYYYMMDD.txt`
- `fixtures/hid/single_actions_YYYYMMDD.txt`
- `fixtures/hid/combo_actions_YYYYMMDD.txt`

## Capture Rules

1. Keep all captures on the same firmware build.
2. Keep the same host OS and app version for one capture set.
3. Save hex report bytes exactly as captured (one report per line).
4. Do not reformat spacing after capture.

## Suggested Action Sets

### `single_actions`

- GREEN/RED/YELLOW/BLUE/ORANGE press and release
- STRUM_UP and STRUM_DOWN transitions
- SELECT, START, GUIDE press and release
- TILT off -> on -> off
- WHAMMY min/mid/max

### `combo_actions`

- Fret + strum combinations
- D-pad diagonals and transition edges
- Multiple fret chord combinations

## Validation

Run:

```bash
cd firmware-rust
python3 -m pip install --user hid
./scripts/capture_hid_fixtures.py --vid 0x6997 --pid 0xB528
./scripts/check_hid_fixtures.sh
```

This verifies required HID files exist and are non-empty.
