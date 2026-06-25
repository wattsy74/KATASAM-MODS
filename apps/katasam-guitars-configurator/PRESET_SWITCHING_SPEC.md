# FEAT-006 Preset Switching Spec

Created: 2026-06-23
Scope: Coordinated app + firmware implementation

## Goal

Add six on-device user preset slots that can be managed from the configurator app and switched on-device during runtime using controller inputs.

## User Interaction

1. User stores colors in one of six slots from the app: User 1..User 6.
2. User long-presses Guide to enter preset switching mode.
3. In preset switching mode:
- D-pad left selects previous slot.
- D-pad right selects next slot.
- Start confirms selected slot as default, applies it, and exits mode.
4. On reboot, device loads and applies the saved default slot.

## Data Model

Store in user_presets.json (existing file):

- User 1..User 6: slot payloads (existing shape)

Add persistent metadata file preset_state.json:

- active_slot: integer 1..6
- default_slot: integer 1..6
- schema_version: 1

Notes:
- Keep user preset payload shape unchanged for backward compatibility.
- If preset_state.json missing/corrupt, firmware defaults both active_slot and default_slot to 1.

## Firmware Requirements

### Input State Machine

States:
- normal
- preset_switch

Transitions:
- normal -> preset_switch: Guide held >= long_press_ms (recommended 900 ms)
- preset_switch -> normal: Start confirm, or Guide long-press cancel timeout path (optional)

Behavior in preset_switch:
- Consume D-pad left/right for slot selection (wraparound 1..6).
- Consume Start for confirm.
- Ignore gameplay actions or gate them until exit.

### Apply/Save Rules

- On left/right: update active_slot and preview slot colors (non-persistent).
- On Start:
- default_slot = active_slot
- persist preset_state.json
- apply selected preset to runtime
- optionally persist config-derived colors if architecture requires
- exit preset_switch

### Visual Feedback

Recommended minimum:
- Flash one LED pattern on mode entry.
- Show selected slot index with a deterministic LED cue.
- Flash confirmation pattern on Start save.

## Serial Command Contract (Recommended)

Add lightweight commands so app can detect/manage feature support:

- READPRESETSTATE
- Response:
- PRESETSTATE:{"active_slot":1,"default_slot":1,"schema_version":1}
- END

- WRITEPRESETSTATE:{json}
- ACK on success, ERROR on failure

- LISTPRESETSLOTS
- Response:
- PRESETSLOTS:["User 1","User 2","User 3","User 4","User 5","User 6"]
- END

Backward compatibility:
- Unknown command should return existing UNKNOWN/ERROR flow.
- App must treat missing command support as legacy firmware and degrade gracefully.

## App Requirements

1. Slot model
- Support User 1..User 6 in dropdown/save validation.
- Preserve compatibility with devices that expose only 5 slots.

2. Capability detection
- On connect, probe READPRESETSTATE with timeout.
- If unsupported, hide switching-mode-specific controls and keep existing preset behavior.

3. UI additions (optional phase)
- Display active/default slot in diagnostics.
- Provide set-default action in app via WRITEPRESETSTATE for users who prefer app-only control.

4. Error handling
- Invalid or missing state file falls back to slot 1 with warning toast.
- Serial command timeout should not block normal editing/preset upload flow.

## Migration Strategy

Phase 1 (done in app):
- Six-slot support in user preset validation/dropdown.

Phase 2 (firmware):
- Add state machine + persistence + optional serial commands.

Phase 3 (app):
- Capability detection + state readout + optional default-slot controls.

## Acceptance Matrix

- Legacy firmware + new app: existing preset features work, no hard errors.
- New firmware + old app: device switching mode works on-device; app still reads/writes preset files.
- New firmware + new app: full FEAT-006 behavior and state visibility.
