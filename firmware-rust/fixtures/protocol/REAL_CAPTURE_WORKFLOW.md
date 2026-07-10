# Real Capture Fixture Workflow

## Naming

Use fixture IDs like `capture-001`, `capture-002`, etc.

For each fixture ID, create:
- `cmd-real-<id>-session.txt`
- `golden-real-<id>-session.txt`

Example:
- `cmd-real-capture-001-session.txt`
- `golden-real-capture-001-session.txt`

## Steps

1. Build command list in `cmd-real-<id>-session.txt`.
2. Capture baseline output from current CircuitPython firmware into `golden-real-<id>-session.txt`.
3. Replay against Rust bridge:

```bash
cd firmware-rust
./scripts/replay_real_capture_fixture.sh <id>
```

If output differs, inspect:
- `fixtures/protocol/out-real-<id>-session.txt`
- `fixtures/protocol/diff-real-<id>-session.txt`

## Goal

Use real capture fixtures as parity gates before enabling console profile work.
