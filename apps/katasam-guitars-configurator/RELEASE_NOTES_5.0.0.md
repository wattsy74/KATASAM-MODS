## KATASAM Configurator v5.0.0

This release rolls up all shipped changes since v4.0.0, including the KATASAM rebrand, major Config Editor consolidation, firmware update improvements, device handling fixes, and color workflow fixes.

### Highlights

- Full KATASAM rebrand across the configurator experience.
- Simpler menu structure with more actions moved into the Config Editor.
- Device rename moved into the Config Editor workflow.
- Improved firmware flashing flow for BOOTSEL devices.
- More reliable device detection, device filtering, and reconnect behavior.
- HEX color normalization throughout config loading, editing, preview, and save.

### What Changed Since v4.0.0

#### Rebrand and UI cleanup

- Updated the configurator branding to KATASAM.
- Applied the refreshed KATASAM visual styling, including the current wordmark and teal-accent UI treatment.
- Reduced top-level menu clutter by moving more device configuration tasks into the Config Editor.

#### Config Editor consolidation

- Added Whammy Calibration directly into the Config Editor.
- Added Device Name editing directly into the Config Editor.
- Removed the Rename Device menu entry in favor of the Config Editor path.
- Added hardware-default detection from the Config Editor defaults action.
- Improved validation for Config Editor device-name input.
- Fixed cases where leaving the Config Editor could hide the Fret and Strum UI.

#### Firmware flashing and BOOTSEL improvements

- Fixed RPI-RP2 BOOTSEL detection on macOS.
- Switched BOOTSEL firmware acquisition to the latest GitHub release, with fallback behavior for offline or failure cases.
- Improved BOOTSEL copy/retry/timeout behavior to prevent stuck flash states.
- Shortened the reconnect path after flashing so follow-up prompts appear sooner.

#### Device detection and connection improvements

- Improved multi-port device filtering so duplicate controller ports are deduplicated correctly.
- Improved device reconnect handling after config writes and reboots.
- Improved pre-connection device-name detection for better device listing and selector behavior.
- Fixed several startup and reconnect paths that could leave the app without an active device.

#### Color and config reliability fixes

- Fixed fret and strum button colors not loading correctly from device config.
- Standardized Config Editor color display and saved config color values to HEX.
- Rewrote tuple or rgb() config colors back to HEX when detected.
- Fixed color wheel fret changes so they persist correctly to config after Apply To Config.
- Preserved correct LED preview behavior after color and preset changes.
- Fixed config order versus UI order mapping for fret colors.

#### Visual polish and UX fixes

- Fixed Apply To Config hover styling so it matches the rest of the app.
- Improved status updates and reconnect feedback across device operations.
- Simplified several modal and in-app flows during setup and configuration.

### Included Completed Work

This release includes the completed work tracked as:

- BUG-001 through BUG-008, excluding BUG-006 which was withdrawn.
- FEAT-001, FEAT-002, FEAT-004, and FEAT-005.

### Known Issues

#### FEAT-003 Post-flash hardware version detection can still misidentify hardware

After firmware flash, the middle-fret hardware detection flow can still produce an incorrect V1 or V2 result in some cases.

Current impact:

- The post-flash prompt appears, but the detected hardware version may be wrong.
- This can lead to the wrong defaults being loaded after flash.

Workaround:

- Verify the selected defaults before applying them to the device.
- If needed, use the Config Editor defaults flow to retry hardware detection.
- If detection still looks wrong, manually apply the correct defaults instead of accepting the post-flash result.

### Upgrade Notes

- App version is now 5.0.0.
- Existing devices and presets remain supported.
- If you previously used the old Rename Device menu action, use the Config Editor Device Name field instead.
