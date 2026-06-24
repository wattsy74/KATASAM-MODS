# KATASAM Configurator Worklist

Created: 2026-06-19
Project: katasam-guitars-configurator

## How We Will Use This

- Work items are grouped into Bugs and Features/Changes.
- Each item has a checkbox so we can track completion.
- Use the Notes and Test Checklist fields to capture progress as we go.

---

## Bugs

### [x] BUG-001 Device picker shows both ports

Priority: High
Status: Done

Issue
- The device picker is showing both ports and is not filtering to show only the second.

Definition of done
- Device list filters correctly and only displays the intended second port.
- No regression for single-device and multi-device environments.

Test checklist
- [x] Verify filtering logic with two available ports.
- [x] Verify behavior when only one port exists.
- [x] Verify behavior when ports connect/disconnect while app is open.

Notes
- 2026-06-19: Added Windows multi-interface dedupe in scan logic to keep highest interface per PNP base (expected to keep second port, e.g. MI_02).
- 2026-06-19: Updated macOS interface selection to keep one best interface per physical device (prefer cu, else highest tty suffix).
- 2026-06-19: User validation confirmed dual-port case now shows only the latter port.

---

### [x] BUG-002 Fret button colours do not show current config on connect

Priority: High
Status: Done

Issue
- When connected, the fret buttons are not showing the current config colours for either pressed or released states.

Definition of done
- On device connect, UI reflects current device config colours for both pressed and released fret states.
- State remains correct after refresh/reconnect.

Test checklist
- [x] Confirm colours load correctly immediately after connection.
- [x] Confirm pressed and released states both map to expected colours.
- [x] Confirm colours remain correct after disconnect/reconnect cycle.

Notes
- 2026-06-19: Found incorrect fret index mapping in renderer applyConfig path (DOM order differs from config order).
- 2026-06-19: Mapping change did not resolve issue and was reverted to reduce regression risk.
- 2026-06-19: Added lenient JSON parsing for serial file reads so config.json can still be parsed when response contains framing/noise; this should allow applyConfig to receive real color arrays instead of leaving defaults.
- 2026-06-19: User reported defaults were written to device; removed forced reload path and added guards to prevent Apply to Config from writing unloaded grey/default UI colours.
- 2026-06-19: Added read-only UI config reload when active device is unchanged but no colour config is loaded; also normalized HEX/RGB tuple/rgb() colours before applying to UI.
- 2026-06-19: Console logs showed config loads successfully but deviceUI.applyConfig was applying RGB arrays directly to CSS. Added RGB tuple normalization there and exposed app-level normalized applyConfig after declaration.
- 2026-06-19: Once colours rendered, confirmed config order (orange, blue, yellow, red, green) differs from UI order (green, red, yellow, blue, orange). Fixed mapping in both applyConfig paths.
- 2026-06-19: Restored physical LED preview after normal preset selection by sending released-state preview commands after UI colour restore; preview command now also accepts RGB tuples defensively.

---

### [x] BUG-003 RPI-RP2 BOOTSEL mode detection not working

Priority: High
Status: Done

Issue
- RPI-RP2 BOOTSEL mode detection is not working, so the app does not offer firmware installation on a new device.

Definition of done
- BOOTSEL mode is detected reliably for supported devices.
- App prompts/offers firmware install flow when a new BOOTSEL device is detected.

Test checklist
- [x] Verify BOOTSEL detection on supported macOS setup.
- [x] Verify firmware install prompt appears on detection.
- [x] Verify non-BOOTSEL devices do not trigger false positives.

Notes
- 2026-06-19: Root cause was Windows-only drive-letter scanning for INFO_UF2.TXT. Added platform-aware detection for macOS /Volumes mount points while preserving Windows behavior.
- 2026-06-19: User validation confirmed BOOTSEL detection now works and firmware prompt appears.

---

### [x] BUG-004 Config editor shows RGB tuple colours

Priority: High
Status: Done

Issue
- Colours are shown as RGB tuples in the Config Editor. All colour display and config storage should use HEX.

Definition of done
- Config Editor displays colour values as HEX strings.
- Loaded configs with RGB tuples are converted to HEX in memory.
- Device `config.json` is rewritten with HEX equivalents when RGB tuples are detected.
- Apply To Config writes HEX colours, not RGB tuples.

Test checklist
- [x] Verify Config Editor colour fields show HEX values.
- [x] Verify loading a tuple-based config rewrites `config.json` with HEX values.
- [x] Verify Apply To Config writes HEX colour arrays.
- [x] Verify UI colour rendering and LED preview still work.

Notes
- 2026-06-19: Added after FEAT-001 validation. Existing code normalized tuple colours for UI rendering but still displayed tuples in the editor and converted HEX back to tuples during Apply To Config.
- 2026-06-19: Added config colour normalization to HEX for loaded config, Config Editor rendering, Config Editor apply, and Apply To Config writes.
- 2026-06-19: MultiDeviceManager now rewrites `config.json` with HEX colours when RGB tuple/rgb() colour values are detected during config reads.

---

### [x] BUG-005 Fret and Strum button UI elements missing

Priority: High
Status: Done

Issue
- Exiting the Config Editor using either the Apply to Session or Back buttons causes the Fret and Strum UI elements to disappear.

Definition of done
- Fret and Strum UI elements remain visible after leaving the Config Editor by any supported exit path.
- The main editor layout returns to its normal non-editor state without requiring an app restart.
- Current colours and selected pressed/released state are preserved when returning to the main UI.

Test checklist
- [x] Open Config Editor, press Back, and visually confirm Fret and Strum elements are visible.
- [x] Open Config Editor, press Apply to Session, and visually confirm Fret and Strum elements are visible.
- [x] Confirm Released/Pressed toggle still switches the visible button set after returning.
- [x] Confirm loaded colours are preserved by the Apply to Session code path.

Notes
- 2026-06-20: Added from user validation pass. Likely related to Config Editor active layout class/display cleanup after hide/apply.
- 2026-06-20: Root cause found: Config Editor buttons reused `.fret-toggle-button`, so the global Released/Pressed toggle handler ran for Back/Apply buttons without `data-state` and hid both Fret/Strum sets.
- 2026-06-20: Fixed by binding the toggle handler only to `.fret-toggle .fret-toggle-button[data-state]` and centralising main Fret/Strum visibility in `syncMainButtonSetVisibility()`.
- 2026-06-20: Static validation passed with `node --check renderer/app.js`; user confirmed the Fret/Strum UI remains visible after Back and Apply to Session, so BUG-005 is resolved.

---

### [x] BUG-006 Reload of device is not happening after Apply Session button is pressed in Config Editor

Priority: High
Status: Withdrawn

Issue
- Exiting the Config Editor using Apply to Session should cause the config write to device and reboot, but the reload is not happening.

Definition of done
- Apply to Session persists the edited config through the intended device write path.
- Device reboots/reloads after the config write and reconnects cleanly.
- UI reloads the updated `config.json` after reconnect.
- Failed writes or reconnect timeouts show a clear error and do not leave stale session state.

Test checklist
- [ ] Change a simple Config Editor field, press Apply to Session, and confirm the device reboots.
- [ ] Confirm the app reconnects after reboot and reloads `config.json`.
- [ ] Confirm the edited field persists after reconnect.
- [ ] Confirm no duplicate or stale device entries appear after reboot/reconnect.
- [ ] Confirm failure path shows a clear error if the device is disconnected before apply.

Notes
- 2026-06-20: Added from user validation pass. Needs careful handling with BUG-005 because apply/exit should restore the main UI while the device write/reboot proceeds.
- 2026-06-20: Withdrawn by user. Current Apply to Session behavior is acceptable, so no code change is required.

---

### [x] BUG-007 Apply to config button hover is Yellow

Priority: Medium
Status: Done

Issue
- Apply To Config button hover colour is yellow; align it with the rest of the styling.

Definition of done
- CSS is corrected so Apply To Config uses the same hover/focus colour treatment as the surrounding action buttons.
- Button text remains readable in normal, hover, focus, and disabled states.

Test checklist
- [x] Hover Apply To Config and visually confirm the hover colour matches the app style.
- [x] Keyboard-focus Apply To Config and confirm the focus state uses readable teal styling.
- [x] Confirm the hidden/visible behavior of Apply To Config is unchanged.

Notes
- 2026-06-20: Added from user visual polish pass.
- 2026-06-20: Changed `#apply-config-btn:hover` from yellow to app teal styling and added matching focus styling.
- 2026-06-22: User validation confirmed the hover styling now matches the app style, so BUG-007 is resolved.

---

### [x] BUG-008 Colour wheel fret changes do not persist to config

Priority: High
Status: Done

Issue
- Selecting a fret colour and changing it with the colour wheel correctly updates the UI and previews on the physical device, but Apply To Config does not write the changed colour into `config.json`. After reboot, the device returns to its previous colour state.

Definition of done
- Colour wheel changes for selected fret buttons update the in-memory config for the correct pressed/released colour array entry.
- Apply To Config writes the changed fret colour to `config.json` as HEX.
- Device reboots into the newly applied colour state after Apply To Config.
- Fret colour mapping remains correct between UI order and config order.
- Strum colour changes and preset-based colour changes continue to persist correctly.

Test checklist
- [x] Select a released fret, change it with the colour wheel, press Apply To Config, and confirm the colour persists after reboot.
- [x] Select a pressed fret, change it with the colour wheel, press Apply To Config, and confirm the colour persists after reboot.
- [x] Confirm the correct fret is updated in `released_color` or `led_color` rather than a neighbouring fret.
- [x] Confirm UI preview and physical LED preview still update immediately while dragging/choosing colour.
- [x] Confirm Apply To Config still writes HEX values.
- [x] Regression check strum colour changes still persist after Apply To Config.

Notes
- 2026-06-20: Added from user report. Likely gap is that colour wheel preview updates the DOM/live preview but does not sync selected fret colour changes back into `originalConfig`/`configDirty` before Apply To Config builds the device payload.
- 2026-06-20: Added explicit selected-button-to-config mapping for `led_color` and `released_color`, and sync colour wheel/HEX input changes into `originalConfig` as colours change. Static diagnostics and `node --check renderer/app.js` passed.
- 2026-06-20: User confirmed colour wheel fret changes now persist after Apply To Config and reboot, so BUG-008 is resolved.

---

## Features / Changes

### [x] FEAT-001 Add Whammy Calibration button in Config Editor

Priority: Medium
Status: Done

Change request
- Add a button in the Whammy section of the Config Editor to open the Whammy Calibration modal.
- After this is implemented, remove Whammy Calibration from the app menu.

Definition of done
- New button is visible in Whammy section and opens existing calibration modal.
- Menu item is removed and no dead routes/actions remain.

Test checklist
- [x] Verify modal opens from new Config Editor button.
- [x] Verify calibration flow still works end-to-end.
- [x] Verify Whammy Calibration no longer appears in menu.

Notes
- 2026-06-19: Added a `Calibrate` action to the Whammy row in the Config Editor that opens the existing Whammy Calibration modal.
- 2026-06-19: Removed the old Whammy Calibration entry from the popup menu and removed its now-unused click handler.
- 2026-06-19: Changed Whammy Calibration Apply so it updates the Config Editor/session values only; it no longer writes `config.json` or reboots immediately. The normal Apply to Session / Apply To Config flow now controls persistence and reboot.
- 2026-06-20: User validation confirmed the modal opens and the updated calibration apply flow works.

---

### [x] FEAT-002 Fetch latest firmware from GitHub for new BOOTSEL devices

Priority: Medium
Status: Done

Change request
- When a new BOOTSEL-mode device is detected, fetch firmware from the latest GitHub release instead of using firmware embedded in the application.

Definition of done
- App resolves latest release firmware artifact from GitHub.
- Firmware download/install flow works for BOOTSEL devices.
- Embedded firmware dependency is removed (or kept only as explicit fallback with clear rules).

Test checklist
- [x] Verify latest release lookup works.
- [x] Verify firmware file download success path.
- [x] Verify behavior for offline/network failure (fallback or clear error).
- [x] Verify install flow with downloaded firmware on BOOTSEL device.

Notes
- 2026-06-19: BOOTSEL flash path now checks the latest `wattsy74/KATASAM-MODS` GitHub release for a UF2 asset before flashing. Asset selection prefers the matching Classic V1/V2 asset when the release contains multiple UF2 variants, with single/generic firmware assets still supported.
- 2026-06-19: Downloaded firmware is cached into the OS temp directory for the current flash. Bundled firmware remains as an explicit offline/network-failure fallback for now.
- 2026-06-19: Latest release currently provides variant assets (`Classic-v1.uf2`, `Classic-v2.uf2`, `Guitarv1.uf2`, `Guitarv2.uf2`). Added V1/V2-aware selection for Classic firmware using `hardware_version.txt` when present, otherwise asking the user to choose.
- 2026-06-19: Hardened BOOTSEL copy step with retry, timeout, filesystem sync on macOS/Linux, and guaranteed flash-state cleanup so the UI cannot remain stuck on "Flashing firmware please wait..." indefinitely.
- 2026-06-19: Console paste showed download completed and hang occurred at copy attempt 1 to `/Volumes/RPI-RP2/Classic-v2.uf2`; switched macOS/Linux copy to a bounded `cp` child process so the timeout can kill the copy instead of waiting on a hung Node filesystem operation.

---

### [x] FEAT-003 User input hardware version detection

Priority: Medium
Status: Done

Change request
- When a device has just been flashed have the user press the middle fret button to ascertain the hardware version and apply the correct default config.

Definition of done
- After a successful flash, the app prompts the user to press the middle fret button for hardware version detection.
- The app detects the hardware version from the button input reliably enough to choose V1 or V2 defaults.
- The correct default config is selected and applied for the detected hardware version.
- If detection fails or times out, the app offers a safe fallback instead of applying the wrong defaults silently.
- The selected/detected hardware version is logged clearly for debugging.

Test checklist
- [x] Flash a device and confirm the hardware version detection prompt appears after reconnect.
- [x] Press the middle fret button on V1 hardware and confirm V1 defaults are selected.
- [x] Press the middle fret button on V2 hardware and confirm V2 defaults are selected.
- [x] Confirm timeout/failure path offers manual choice or cancellation.
- [x] Confirm applied defaults persist after config write/reboot.
- [x] Confirm detection does not trigger during normal app startup unless the device was just flashed.

Notes
- 2026-06-20: Needs integration point after FEAT-002 flash/reconnect flow. Middle fret should be treated as the hardware-version signal source before default config selection.
- 2026-06-20: Added shared hardware-defaults detection flow and wired it into post-flash reconnect/auto-connect path. It now prompts for middle-fret hardware confirmation, applies V1/V2 defaults to session, and exposes Apply To Config for persistence.
- 2026-06-20: Timing of the prompt was 90sec plus, shortened but now in between the initial setup and the determination was wrong at that point said v1 hardware when it should be v2
- 2026-06-24: Fixed timing bug (part 1) - removed unreliable YELLOW_FRET fallback, now only trusts LEFT (V2) and TILT (V1) hardware pin states; added 500ms stabilization delay
- 2026-06-24: Fixed double-reboot timing issue (part 2) - keep polling through device reboot cycle, trigger detection only on second reconnect after 1500ms config load time
- 2026-06-24: Fixed polling stop issue (part 3) - polling was stopping too early on first connect, preventing detection of second reboot; now continues polling through full reboot cycle with extended 15s timeout
- 2026-06-24: Finalized FEAT-003 as Done. Post-flash detection now reuses the working diagnostics polling path, waits for device files/config readiness, delays modal by 10s in post-flash context, suppresses repeat BOOTSEL prompt for 60s after user confirmation, and auto-applies detected defaults to config with reboot/reload.

---

### [x] FEAT-004 User input hardware version detection when defaults button pressed in config editor

Priority: Medium
Status: Done

Change request
- When a defaults button is pressed in Config Editor, have the user press the middle fret button to ascertain the hardware version and apply the correct default config.

Definition of done
- Config Editor defaults action can detect hardware version from middle fret input before choosing defaults.
- Correct V1 or V2 default config is loaded into the editor/session based on detected hardware version.
- Manual V1/V2 default buttons remain available as fallback if detection fails or user cancels.
- Detection flow does not write to device until the user applies the resulting config through the normal apply path.

Test checklist
- [x] Press the defaults action in Config Editor and confirm the hardware detection prompt appears.
- [x] Press the middle fret button on V1 hardware and confirm V1 defaults load into the editor.
- [x] Press the middle fret button on V2 hardware and confirm V2 defaults load into the editor.
- [x] Confirm cancelling detection does not change the current editor values.
- [x] Confirm Apply to Session/Config is still required before device config is changed.

Notes
- 2026-06-20: Related to FEAT-003, but scoped to Config Editor defaults rather than post-flash onboarding.
- 2026-06-20: Added `Detect Hardware Defaults` action in Config Editor and wired it to the same shared V1/V2 detection flow used by FEAT-003.

---

### [x] FEAT-005 Change device name in Config Editor

Priority: Medium
Status: Done

Change request
- Show the device name field in Config Editor and remove the Rename Device menu item and modal and warnings, this is a much simpler process and doesn't carry the risks the previous mechanism did.

Definition of done
- Device Name field is visible and editable in the Config Editor.
- Input validation is in place for empty, overlong, and invalid-character names.
- Device renames through the same config save/reboot flow used by Apply to Session.
- Rename Device menu item is removed.
- Legacy rename modal/warnings are no longer reachable from the menu path.
- Header/footer/device selector labels update after reconnect using the new name.

Test checklist
- [x] Open Config Editor and confirm Device Name is visible and editable.
- [x] Try an empty name and confirm validation prevents apply.
- [x] Try an overlong or invalid-character name and confirm validation prevents apply.
- [x] Change to a valid name, apply, and confirm device writes/reboots.
- [x] Confirm the new name appears after reconnect in header/footer/device selector UI.
- [x] Confirm Rename Device is no longer present in the popup menu.
- [x] Confirm the old rename modal/warnings are no longer reachable from the menu path.

Notes
- 2026-06-20: Intended to replace the old Rename Device flow with the safer Config Editor config-save path.
- 2026-06-22: Restarted FEAT-005 from the current clean renderer state. Added `device_name` to Config Editor with validation, and removed only the Rename Device menu item to avoid destabilizing the legacy modal code.
- 2026-06-22: User validation completed successfully, so FEAT-005 is done with the menu-only removal scope.

---

### [ ] FEAT-006 On-device switchable user presets (6 slots)

Priority: High
Status: In Progress

Change request
- Add six on-device user preset slots that can be populated from the app and saved on the controller.
- Add firmware-side preset switching mode entered by long-pressing Guide.
- In switching mode, D-pad left/right cycles through the six slots, and Start sets the selected slot as default and exits switching mode.

Definition of done
- App can read/write all six user preset slots (`User 1` to `User 6`) to device storage.
- Firmware supports long-press Guide to enter/exit preset switching mode.
- Firmware supports D-pad left/right slot navigation with visible feedback.
- Firmware supports Start to persist selected slot as default and apply it.
- Default selected slot is restored on boot and reflected in runtime behavior.
- Backward compatibility: devices without FEAT-006 firmware continue to work with existing preset flows.

Test checklist
- [ ] Save colors from app into each slot `User 1` through `User 6` and verify values persist after reconnect.
- [ ] Enter switching mode using Guide long-press and confirm normal gameplay input is gated while active.
- [ ] Press D-pad left/right in switching mode and confirm slot index changes with wraparound.
- [ ] Press Start in switching mode and confirm selected slot is applied and saved as default.
- [ ] Reboot device and verify last saved default slot is automatically active.
- [ ] Verify legacy firmware (without switching commands) does not crash app and shows graceful fallback behavior.

Notes
- 2026-06-23: Initial scope captured from user request; requires coordinated firmware + configurator changes.
- 2026-06-23: App-side preparation started by enabling six user slots in renderer validation/dropdown handling.
- 2026-06-24: Replaced ambiguous user preset dropdown/update flow with six dedicated slot buttons (`1..6`) in a radio-style row.
- 2026-06-24: Implemented explicit slot interactions in app UI: short click loads slot, long-press (700ms) saves current colours to that slot.
- 2026-06-24: Added clarity and safety UX updates: selecting a user slot resets factory preset dropdown to `Select preset...`, and slot buttons remain disabled until a device is connected.

---

## Workflow

- Move `Status` from `Todo` -> `In Progress` -> `Done` as we work.
- Add dated notes under each item while investigating.
- Keep this file as the single source of truth for this set of changes.
