## 🚀 KATASAM Configurator v5.0.0

This release bundles all shipped changes since v4.0.0, including the KATASAM rebrand, Config Editor consolidation, firmware update improvements, device handling fixes, and color/config reliability fixes.

### Highlights

- Full KATASAM rebrand across the configurator UI
- Reduced menu clutter and moved more actions into the Config Editor
- Device rename now handled from the Config Editor workflow
- Improved BOOTSEL detection and latest-release firmware flashing
- Better device filtering, reconnect handling, and color/config persistence

### Included Improvements

- Added Whammy Calibration to the Config Editor
- Added Device Name editing to the Config Editor
- Removed the Rename Device menu entry
- Added hardware-default detection from the Config Editor defaults action
- Fixed duplicate device-port picker issues
- Fixed fret and strum colors not loading correctly on connect
- Standardized config/editor/device color handling to HEX
- Fixed color wheel fret changes so they persist after Apply To Config
- Fixed Config Editor exit paths that could hide the Fret and Strum UI
- Fixed Apply To Config hover styling to match the app theme
- Improved BOOTSEL detection on macOS
- Switched firmware flashing to use the latest GitHub release with fallback handling
- Improved BOOTSEL copy/retry/timeout handling to prevent stuck flash states

### Known Issue

**Post-flash hardware detection can still misidentify V1 vs V2 hardware.**

After firmware flash, the middle-fret hardware detection flow may still choose the wrong hardware version in some cases.

Workaround:
- Verify the detected defaults before applying them
- Retry detection from the Config Editor defaults flow if needed
- Manually choose the correct defaults if the detected result looks wrong

### Notes

- App version is now 5.0.0
- Existing devices and presets remain supported
- If you previously used the old Rename Device menu action, use the Device Name field in the Config Editor instead
