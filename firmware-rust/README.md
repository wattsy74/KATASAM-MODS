# KATASAM Firmware Rust Workspace

This workspace prepares the migration from CircuitPython to Rust.

## Migration Rules

- Keep full compatibility with the KATASAM Configurator protocol.
- Keep existing config schema behavior backward compatible.
- Reach PC parity first (Clone Hero, YARG, FNF) before console-specific profiles.
- Add console support as profile adapters, not core rewrites.

## Workspace Layout

- crates/firmware-core: Input model and profile interface contracts
- crates/config-protocol: Serial protocol parsing/formatting for configurator compatibility
- crates/config-runtime: Host-side protocol execution harness for parity testing
- crates/platform-profiles: PC and console profile adapters
- crates/board-rp2040: Board-facing abstractions for GPIO/ADC/timers/reboot paths
- app/main: Integration entry point

## Toolchain Setup (macOS)

1. Install rustup:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Add embedded target:
   rustup target add thumbv6m-none-eabi
3. Optional flash tooling:
   cargo install probe-rs-tools

## First Milestone

- Implement configurator protocol parity and PC profile parity in Rust.
- Validate no regressions before enabling console profiles.

## Protocol Replay Workflow

Use the fixture replay script to run command traces through the stdio bridge and compare output against golden files:

```bash
cd firmware-rust
./scripts/replay_protocol_fixtures.sh
```

Fixtures:
- Commands: `fixtures/protocol/cmd-*-session.txt`
- Expected output: `fixtures/protocol/golden-*-session.txt`
- Actual output: `fixtures/protocol/out-*-session.txt`

For real hardware captures, use:

```bash
cd firmware-rust
./scripts/replay_real_capture_fixture.sh capture-001
```

See `fixtures/protocol/REAL_CAPTURE_WORKFLOW.md` for naming and intake steps.

## HID Capture Workflow

HID parity is tracked with fixture files under `fixtures/hid/`.

Run the fixture presence/data check:

```bash
cd firmware-rust
python3 -m pip install --user hid
./scripts/capture_hid_fixtures.py --vid 0x6997 --pid 0xB528
./scripts/check_hid_fixtures.sh
./scripts/analyze_hid_fixtures.py
```

See `fixtures/hid/HID_CAPTURE_WORKFLOW.md` for capture expectations.

## RP2040 Test Firmware (HID + BOOTSEL)

Build a flashable UF2 for the Rust RP2040 test firmware:

```bash
cd firmware-rust
./scripts/build_test_fw_uf2.sh
```

Output:
- `dist/katasam-rp2040-test-fw-w25q080.uf2` (default copy target)
- `dist/katasam-rp2040-test-fw-is25lp080.uf2`
- `dist/katasam-rp2040-test-fw-generic03h.uf2`
- `dist/katasam-rp2040-test-fw.uf2` (copied from `-w25q080`)

If a board immediately returns to BOOTSEL after flashing, try the boot2 variants in this order:
1. `katasam-rp2040-test-fw-w25q080.uf2`
2. `katasam-rp2040-test-fw-is25lp080.uf2`
3. `katasam-rp2040-test-fw-generic03h.uf2`

Behavior in this test firmware:
- Enumerates as a USB HID gamepad and continuously streams changing XY/button reports.
- Exposes USB CDC serial control command for software BOOTSEL entry.

Programmatic BOOTSEL entry (no button access required):

```bash
cd firmware-rust
python3 scripts/enter_bootsel.py --port /dev/cu.usbmodemXXXX
```

If `--port` is omitted, the script auto-detects serial devices and prefers the test firmware VID/PID.
The default command sent is `KATASAM_BOOTSEL_V1`.

List candidate serial ports:

```bash
cd firmware-rust
python3 scripts/enter_bootsel.py --list-ports
```

For older test firmware builds that still use legacy command tokens:

```bash
cd firmware-rust
python3 scripts/enter_bootsel.py --port /dev/cu.usbmodemXXXX --command BOOTSEL
```

Quick HID stream verification after flashing:

```bash
cd firmware-rust
python3 scripts/verify_hid_stream.py --vid 0x2E8A --pid 0x1031 --duration 5
```

The verifier checks report count, uniqueness, and per-byte activity thresholds and returns non-zero on failure.
