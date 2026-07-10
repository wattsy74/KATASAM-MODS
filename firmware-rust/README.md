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
