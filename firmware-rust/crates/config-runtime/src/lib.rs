use config_protocol::{
    filename_from_path, Command, LineOutcome, OutboundMessage, ParserMode, ProtocolAction,
    ProtocolEngine,
};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

const DEFAULT_DEVICE_NAME: &str = "Guitar Controller";
const RUNTIME_VERSION: &str = "0.1.0";
const MOCK_UID: &str = "RUSTSIM-0001";

#[derive(Default)]
pub struct Runtime {
    engine: ProtocolEngine,
    files: BTreeMap<String, String>,
    identity_version: String,
    identity_device_name: Option<String>,
    identity_uid: String,
    whammy_value: i32,
    joystick_x: i32,
    joystick_y: i32,
    pin_reads: BTreeMap<String, String>,
    active_write_path: Option<String>,
    write_is_streaming: bool,
    write_lines: Vec<String>,
    merge_lines: Vec<String>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            identity_version: RUNTIME_VERSION.to_string(),
            identity_device_name: None,
            identity_uid: MOCK_UID.to_string(),
            whammy_value: -1,
            joystick_x: -1,
            joystick_y: -1,
            pin_reads: BTreeMap::new(),
            ..Self::default()
        }
    }

    pub fn with_identity(mut self, version: &str, device_name: &str, uid: &str) -> Self {
        if !version.trim().is_empty() {
            self.identity_version = version.trim().to_string();
        }
        if !device_name.trim().is_empty() {
            self.identity_device_name = Some(device_name.trim().to_string());
        }
        if !uid.trim().is_empty() {
            self.identity_uid = uid.trim().to_string();
        }
        self
    }

    pub fn with_controls(mut self, whammy_value: i32, joystick_x: i32, joystick_y: i32) -> Self {
        self.whammy_value = whammy_value;
        self.joystick_x = joystick_x;
        self.joystick_y = joystick_y;
        self
    }

    pub fn with_pin_read(mut self, name: &str, value: &str) -> Self {
        let key = name.trim();
        let val = value.trim();
        if !key.is_empty() && !val.is_empty() {
            self.pin_reads.insert(key.to_string(), val.to_string());
        }
        self
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.insert(normalize_path(path), content.to_string());
        self
    }

    pub fn file_content(&self, path: &str) -> Option<&str> {
        self.files.get(&normalize_path(path)).map(String::as_str)
    }

    pub fn process_line(&mut self, line: &str) -> Vec<String> {
        let outcome = self.engine.process_line(line);
        let mut out = Vec::new();

        if outcome.send_ack {
            out.push(render_message(OutboundMessage::Ack { line: line.trim() }));
        }

        match outcome.action {
            ProtocolAction::Command(cmd) => self.handle_command(cmd, &mut out),
            ProtocolAction::WriteChunk { line } => self.write_lines.push(line.to_string()),
            ProtocolAction::WriteCommit => self.commit_write(&mut out),
            ProtocolAction::MergeChunk { line } => self.merge_lines.push(line.to_string()),
            ProtocolAction::MergeCommit => self.commit_merge(&mut out),
        }

        out
    }

    fn handle_command(&mut self, cmd: Command<'_>, out: &mut Vec<String>) {
        match cmd {
            Command::Ready => out.push("FIRMWARE_READY:OK\n".to_string()),
            Command::ReadVersion => out.push(format!("VERSION:{}\nEND\n", self.identity_version)),
            Command::ReadDeviceName => {
                let name = self.read_device_name();
                out.push(format!("{}\nEND\n", name));
            }
            Command::ReadUid => out.push(format!("{}\nEND\n", self.identity_uid)),
            Command::ReadFile { path } => self.read_file(path, out),
            Command::WriteFile { path } => self.start_write(path, out),
            Command::ImportUser => {
                self.active_write_path = Some("/user_presets.json".to_string());
                self.merge_lines.clear();
            }
            Command::Mkdir { path } => {
                let p = path.trim();
                if p.is_empty() {
                    out.push("MKDIR:ERROR:empty path\n".to_string());
                } else {
                    out.push(format!("MKDIR:SUCCESS:{}\n", p));
                }
            }
            Command::Reboot => out.push("Rebooting...\n".to_string()),
            Command::RebootBootsel => out.push(" Rebooting to BOOTSEL mode...\n".to_string()),
            Command::ReadWhammy => out.push(format!("WHAMMY:{}\n", self.whammy_value)),
            Command::ReadJoystick => {
                out.push(format!("JOYSTICK:X:{}:Y:{}\n", self.joystick_x, self.joystick_y))
            }
            Command::ReadPin { name } => {
                let pin_value = self.pin_reads.get(name).map_or("ERR", String::as_str);
                out.push(format!("PIN:{}:{}\n", name, pin_value));
            }
            Command::SetLedHex { led, .. } => out.push(format!("SETLED:{}:OK\n", led)),
            Command::SetLedRgb { index, .. } => out.push(format!("SETLED:{}:OK\n", index)),
            Command::LedRestore => out.push("LEDRESTORE:OK\n".to_string()),
            Command::TiltWave => out.push("TILTWAVE:STARTED\n".to_string()),
            Command::TiltWaveEnable { enabled } => out.push(format!("TILTWAVE_ENABLE:{}\n", enabled)),
            Command::DetectPin { name } => {
                out.push(format!("PINDETECT:START:{}\n", name));
                out.push(format!("PINDETECT:NONE:{}\n", name));
            }
            Command::SavePin { name, pin } => out.push(format!("PINDETECT:SAVED:{}:{}\n", name, pin)),
            Command::CancelPinDetect => out.push("PINDETECT:CANCELLED\n".to_string()),
            Command::PreviewLed { .. } => {}
            Command::Demo => out.push("DEMO:STARTED\n".to_string()),
            Command::End => {}
            Command::Unknown => out.push(render_message(OutboundMessage::UnknownCommand)),
        }
    }

    fn read_file(&self, path: &str, out: &mut Vec<String>) {
        let normalized = normalize_path(path);
        let filename = filename_from_path(&normalized);
        out.push(render_message(OutboundMessage::StartFile { filename }));

        if let Some(content) = self.files.get(&normalized) {
            let trimmed = content.trim();
            if !trimmed.is_empty() && trimmed != "FIRMWARE_READY:OK" {
                out.push(content.clone());
            }
        } else {
            out.push(format!("ERROR: File not found: {}\n", normalized));
        }

        out.push(render_message(OutboundMessage::EndFile { filename }));
    }

    fn start_write(&mut self, path: &str, out: &mut Vec<String>) {
        let normalized = normalize_path(path);
        let filename = filename_from_path(&normalized).to_string();
        let is_stream = self.engine.mode() == ParserMode::WriteStream;
        self.active_write_path = Some(normalized);
        self.write_is_streaming = is_stream;
        self.write_lines.clear();

        out.push(render_message(OutboundMessage::WriteFileReady {
            filename: &filename,
        }));
        if is_stream {
            out.push(render_message(OutboundMessage::StreamReady {
                filename: &filename,
            }));
        }
    }

    fn commit_write(&mut self, out: &mut Vec<String>) {
        let Some(path) = self.active_write_path.take() else {
            out.push("ERROR: Failed to write <unknown>: missing write target\n".to_string());
            self.write_lines.clear();
            self.write_is_streaming = false;
            return;
        };

        let raw = self.write_lines.join("\n");
        self.write_lines.clear();
        let is_stream = self.write_is_streaming;
        self.write_is_streaming = false;

        if path.ends_with(".json") {
            let parsed = match serde_json::from_str::<Value>(&raw) {
                Ok(v) => v,
                Err(err) => {
                    out.push(format!("ERROR: Failed to write {}: {}\n", path, err));
                    return;
                }
            };

            if path == "/user_presets.json" {
                let Value::Object(obj) = parsed else {
                    out.push("ERROR: Invalid user_presets.json structure, write rejected\n".to_string());
                    return;
                };

                if !is_valid_user_presets(&obj) {
                    out.push("ERROR: Invalid user_presets.json structure, write rejected\n".to_string());
                    return;
                }
            }

            let mut persisted = raw;
            if !persisted.ends_with('\n') {
                persisted.push('\n');
            }
            self.files.insert(path.clone(), persisted);

            if path == "/config.json" {
                out.push("File /config.json written (atomic)\n".to_string());
            } else {
                out.push(format!("File {} written (atomic)\n", path));
            }
            return;
        }

        let mut persisted = raw;
        if !persisted.ends_with('\n') {
            persisted.push('\n');
        }
        self.files.insert(path.clone(), persisted);

        if is_stream {
            out.push(format!("File {} written (high-speed streaming)\n", path));
        } else {
            out.push(render_message(OutboundMessage::FileWritten { path: &path }));
        }
    }

    fn commit_merge(&mut self, out: &mut Vec<String>) {
        let target = "/user_presets.json";
        let raw = self.merge_lines.join("\n");
        self.merge_lines.clear();

        let new_value = match serde_json::from_str::<Value>(&raw) {
            Ok(v) => v,
            Err(err) => {
                out.push(format!("ERROR: {}\n", err));
                return;
            }
        };

        let Value::Object(new_obj) = new_value else {
            out.push("ERROR: Invalid user_presets.json structure, merge rejected\n".to_string());
            return;
        };

        let existing = self
            .files
            .get(target)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .unwrap_or_else(|| Value::Object(Map::new()));

        let Value::Object(mut merged) = existing else {
            out.push("ERROR: Invalid user_presets.json structure, merge rejected\n".to_string());
            return;
        };

        for (k, v) in new_obj {
            merged.insert(k, v);
        }

        if !is_valid_user_presets(&merged) {
            out.push("ERROR: Invalid user_presets.json structure, merge rejected\n".to_string());
            return;
        }

        let mut s = Value::Object(merged).to_string();
        s.push('\n');
        self.files.insert(target.to_string(), s);
        out.push("Merged into /user_presets.json (atomic)\n".to_string());
    }

    fn read_device_name(&self) -> String {
        if let Some(name) = &self.identity_device_name {
            return name.clone();
        }

        self.files
            .get("/config.json")
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| {
                v.get("device_name")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| DEFAULT_DEVICE_NAME.to_string())
    }
}

fn normalize_path(path: &str) -> String {
    let p = path.trim();
    if p.starts_with('/') {
        p.to_string()
    } else {
        format!("/{}", p)
    }
}

fn is_valid_user_presets(map: &Map<String, Value>) -> bool {
    map.iter().all(|(k, v)| {
        let key = k.to_ascii_lowercase();
        let key_ok = key.starts_with("user ") || key.contains("preset");
        key_ok && matches!(v, Value::Object(_))
    })
}

fn render_message(msg: OutboundMessage<'_>) -> String {
    let mut bytes = [0u8; 160];
    let n = msg.write_into(&mut bytes);
    String::from_utf8_lossy(&bytes[..n]).to_string()
}

fn _consume_line_outcome(_outcome: LineOutcome<'_>) {}

#[cfg(test)]
mod tests {
    use super::Runtime;

    fn run_session(rt: &mut Runtime, commands: &[&str]) -> Vec<String> {
        let mut out = Vec::new();
        for cmd in commands {
            out.extend(rt.process_line(cmd));
        }
        out
    }

    #[test]
    fn writefile_and_readfile_roundtrip() {
        let mut rt = Runtime::new();

        let r1 = rt.process_line("WRITEFILE:/config.json");
        assert!(r1.iter().any(|s| s == "WRITEFILE:READY:config.json\n"));

        let _ = rt.process_line("{\"device_name\":\"KATASAM Test\"}");
        let r2 = rt.process_line("END");
        assert!(r2.iter().any(|s| s == "File /config.json written (atomic)\n"));

        let r3 = rt.process_line("READFILE:/config.json");
        assert!(
            r3.first()
                .is_some_and(|s| s.starts_with("ACK: READFILE:/config.jso"))
        );
        assert!(r3.iter().any(|s| s == "START_config.json\n"));
        assert!(r3.iter().any(|s| s.contains("device_name")));
        assert!(r3.iter().any(|s| s == "END_config.json\n"));
    }

    #[test]
    fn importuser_merges_into_existing_file() {
        let existing = "{\"User 1\":{\"GREEN_FRET Released\":\"#00FF00\"}}\n";
        let mut rt = Runtime::new().with_file("/user_presets.json", existing);

        let _ = rt.process_line("IMPORTUSER");
        let _ = rt.process_line("{\"User 2\":{\"RED_FRET Released\":\"#FF0000\"}}");
        let out = rt.process_line("END");

        assert!(out.iter().any(|s| s == "Merged into /user_presets.json (atomic)\n"));

        let merged = rt.file_content("/user_presets.json").unwrap_or("");
        assert!(merged.contains("User 1"));
        assert!(merged.contains("User 2"));
    }

    #[test]
    fn write_user_presets_rejects_invalid_shape() {
        let mut rt = Runtime::new();
        let _ = rt.process_line("WRITEFILE:/user_presets.json");
        let _ = rt.process_line("{\"bad\":123}");
        let out = rt.process_line("END");
        assert!(
            out.iter()
                .any(|s| s == "ERROR: Invalid user_presets.json structure, write rejected\n")
        );
    }

    #[test]
    fn writefile_streaming_reports_high_speed_completion() {
        let mut rt = Runtime::new();
        let r1 = rt.process_line("WRITEFILE:/code.py");
        assert!(r1.iter().any(|s| s == "WRITEFILE:READY:code.py\n"));
        assert!(r1.iter().any(|s| s == "STREAM:READY:code.py\n"));

        let _ = rt.process_line("print('hi')");
        let r2 = rt.process_line("END");
        assert!(
            r2.iter()
                .any(|s| s == "File /code.py written (high-speed streaming)\n")
        );
    }

    #[test]
    fn readfile_missing_reports_error_and_end_marker() {
        let mut rt = Runtime::new();
        let out = rt.process_line("READFILE:/missing.json");

        assert!(
            out.first()
                .is_some_and(|s| s.starts_with("ACK: READFILE:/missing.js"))
        );
        assert!(out.iter().any(|s| s == "START_missing.json\n"));
        assert!(
            out.iter()
                .any(|s| s == "ERROR: File not found: /missing.json\n")
        );
        assert!(out.iter().any(|s| s == "END_missing.json\n"));
    }

    #[test]
    fn golden_basic_session_output_matches_fixture() {
        let mut rt = Runtime::new();
        let commands = [
            "READY?",
            "READVERSION",
            "READUID",
            "WRITEFILE:/config.json",
            "{\"device_name\":\"KATASAM Gold\"}",
            "END",
            "READDEVICENAME",
            "READFILE:/config.json",
            "UNKNOWN_CMD",
            "MKDIR:/updates",
        ];

        let actual = run_session(&mut rt, &commands).concat();
        let expected = include_str!("../../../fixtures/protocol/golden-basic-session.txt");
        assert_eq!(actual, expected);
    }

    #[test]
    fn golden_control_session_output_matches_fixture() {
        let mut rt = Runtime::new();
        let commands = [
            "READWHAMMY",
            "READJOYSTICK",
            "READPIN:GREEN_FRET",
            "DETECTPIN:GREEN_FRET",
            "SAVEPIN:GREEN_FRET:GP1",
            "CANCELPINDETECT",
            "TILTWAVE_ENABLE:ON",
            "TILTWAVE",
            "SETLED:1:2:3:4",
            "LEDRESTORE",
            "DEMO",
            "REBOOT",
            "REBOOTBOOTSEL",
        ];

        let actual = run_session(&mut rt, &commands).concat();
        let expected = include_str!("../../../fixtures/protocol/golden-control-session.txt");
        assert_eq!(actual, expected);
    }

    #[test]
    fn golden_failure_session_output_matches_fixture() {
        let mut rt = Runtime::new();
        let commands = [
            "READFILE:/missing.json",
            "WRITEFILE:/user_presets.json",
            "{\"bad\":123}",
            "END",
            "IMPORTUSER",
            "{\"bad\":123}",
            "END",
            "MKDIR:",
        ];

        let actual = run_session(&mut rt, &commands).concat();
        let expected = include_str!("../../../fixtures/protocol/golden-failure-session.txt");
        assert_eq!(actual, expected);
    }
}
