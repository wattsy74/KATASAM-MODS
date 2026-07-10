#![cfg_attr(not(test), no_std)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Ready,
    ReadVersion,
    ReadDeviceName,
    ReadUid,
    ReadFile { path: &'a str },
    WriteFile { path: &'a str },
    ImportUser,
    ReadPin { name: &'a str },
    ReadWhammy,
    ReadJoystick,
    DetectPin { name: &'a str },
    SavePin { name: &'a str, pin: &'a str },
    CancelPinDetect,
    Demo,
    PreviewLed { led: &'a str, color: &'a str },
    SetLedHex { led: &'a str, color: &'a str },
    SetLedRgb { index: u8, r: u8, g: u8, b: u8 },
    LedRestore,
    TiltWave,
    TiltWaveEnable { enabled: bool },
    Mkdir { path: &'a str },
    Reboot,
    RebootBootsel,
    End,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    Buffered,
    Streaming,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolAction<'a> {
    Command(Command<'a>),
    WriteChunk { line: &'a str },
    WriteCommit,
    MergeChunk { line: &'a str },
    MergeCommit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineOutcome<'a> {
    pub action: ProtocolAction<'a>,
    pub send_ack: bool,
}

pub struct ProtocolEngine {
    mode: ParserMode,
}

impl ProtocolEngine {
    pub const fn new() -> Self {
        Self {
            mode: ParserMode::Idle,
        }
    }

    pub const fn mode(&self) -> ParserMode {
        self.mode
    }

    pub fn process_line<'a>(&mut self, line: &'a str) -> LineOutcome<'a> {
        let trimmed = line.trim();

        match self.mode {
            ParserMode::Idle => {
                let cmd = parse_command(trimmed);

                if let Command::WriteFile { path } = cmd {
                    self.mode = match select_write_mode(path) {
                        WriteMode::Buffered => ParserMode::Write,
                        WriteMode::Streaming => ParserMode::WriteStream,
                    };
                } else if matches!(cmd, Command::ImportUser) {
                    self.mode = ParserMode::MergeUser;
                }

                let send_ack = requires_ack(&cmd);
                LineOutcome {
                    action: ProtocolAction::Command(cmd),
                    send_ack,
                }
            }
            ParserMode::Write | ParserMode::WriteStream => {
                if trimmed == "END" {
                    self.mode = ParserMode::Idle;
                    LineOutcome {
                        action: ProtocolAction::WriteCommit,
                        send_ack: false,
                    }
                } else {
                    LineOutcome {
                        action: ProtocolAction::WriteChunk { line: trimmed },
                        send_ack: false,
                    }
                }
            }
            ParserMode::MergeUser => {
                if trimmed == "END" {
                    self.mode = ParserMode::Idle;
                    LineOutcome {
                        action: ProtocolAction::MergeCommit,
                        send_ack: false,
                    }
                } else {
                    LineOutcome {
                        action: ProtocolAction::MergeChunk { line: trimmed },
                        send_ack: false,
                    }
                }
            }
        }
    }
}

impl Default for ProtocolEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutboundMessage<'a> {
    Ack { line: &'a str },
    StartFile { filename: &'a str },
    EndFile { filename: &'a str },
    WriteFileReady { filename: &'a str },
    StreamReady { filename: &'a str },
    FileWritten { path: &'a str },
    UnknownCommand,
}

impl<'a> OutboundMessage<'a> {
    pub fn write_into(self, dst: &mut [u8]) -> usize {
        match self {
            OutboundMessage::Ack { line } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"ACK: ");
                n += push_bytes(dst, n, ack_head_20(line).as_bytes());
                n += push_bytes(dst, n, b"\n");
                n
            }
            OutboundMessage::StartFile { filename } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"START_");
                n += push_bytes(dst, n, filename.as_bytes());
                n += push_bytes(dst, n, b"\n");
                n
            }
            OutboundMessage::EndFile { filename } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"END_");
                n += push_bytes(dst, n, filename.as_bytes());
                n += push_bytes(dst, n, b"\n");
                n
            }
            OutboundMessage::WriteFileReady { filename } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"WRITEFILE:READY:");
                n += push_bytes(dst, n, filename.as_bytes());
                n += push_bytes(dst, n, b"\n");
                n
            }
            OutboundMessage::StreamReady { filename } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"STREAM:READY:");
                n += push_bytes(dst, n, filename.as_bytes());
                n += push_bytes(dst, n, b"\n");
                n
            }
            OutboundMessage::FileWritten { path } => {
                let mut n = 0;
                n += push_bytes(dst, n, b"File ");
                n += push_bytes(dst, n, path.as_bytes());
                n += push_bytes(dst, n, b" written\n");
                n
            }
            OutboundMessage::UnknownCommand => push_bytes(dst, 0, b"ERROR: Unknown command\n"),
        }
    }
}

pub fn ack_head_20(line: &str) -> &str {
    let max = 20;
    if line.len() <= max {
        return line;
    }

    let mut end = 0;
    for (idx, _) in line.char_indices() {
        if idx <= max {
            end = idx;
        } else {
            break;
        }
    }

    if end == 0 {
        ""
    } else {
        &line[..end]
    }
}

pub fn filename_from_path(path: &str) -> &str {
    let trimmed = path.trim();
    if let Some((_, tail)) = trimmed.rsplit_once('/') {
        if tail.is_empty() {
            trimmed
        } else {
            tail
        }
    } else {
        trimmed
    }
}

pub fn select_write_mode(path: &str) -> WriteMode {
    let trimmed = path.trim();
    let b = trimmed.as_bytes();
    let is_py = b.len() >= 3 && b[b.len() - 3..].eq_ignore_ascii_case(b".py");

    let stream_targets = [
        "serial_handler.py",
        "code.py",
        "gamepad.py",
        "hardware.py",
        "utils.py",
        "demo_routine.py",
        "demo_state.py",
        "pin_detect.py",
        "boot.py",
    ];

    if stream_targets
        .iter()
        .any(|s| contains_ascii_nocase(trimmed.as_bytes(), s.as_bytes()))
        || (is_py && trimmed.len() > 8)
    {
        WriteMode::Streaming
    } else {
        WriteMode::Buffered
    }
}

pub fn parse_command(line: &str) -> Command<'_> {
    let trimmed = line.trim();

    if trimmed == "FIRMWARE_READY?" || trimmed == "READY?" {
        Command::Ready
    } else if trimmed == "READVERSION" {
        Command::ReadVersion
    } else if trimmed == "READDEVICENAME" {
        Command::ReadDeviceName
    } else if trimmed == "READUID" {
        Command::ReadUid
    } else if let Some(path) = trimmed.strip_prefix("READFILE:") {
        Command::ReadFile { path }
    } else if let Some(path) = trimmed.strip_prefix("WRITEFILE:") {
        Command::WriteFile { path }
    } else if trimmed == "IMPORTUSER" {
        Command::ImportUser
    } else if let Some(name) = trimmed.strip_prefix("READPIN:") {
        Command::ReadPin { name }
    } else if trimmed == "READWHAMMY" {
        Command::ReadWhammy
    } else if trimmed == "READJOYSTICK" {
        Command::ReadJoystick
    } else if let Some(name) = trimmed.strip_prefix("DETECTPIN:") {
        Command::DetectPin { name }
    } else if let Some(rest) = trimmed.strip_prefix("SAVEPIN:") {
        let mut parts = rest.split(':');
        match (parts.next(), parts.next()) {
            (Some(name), Some(pin)) if !name.is_empty() && !pin.is_empty() => {
                Command::SavePin { name, pin }
            }
            _ => Command::Unknown,
        }
    } else if trimmed == "CANCELPINDETECT" {
        Command::CancelPinDetect
    } else if trimmed == "DEMO" {
        Command::Demo
    } else if let Some(rest) = trimmed.strip_prefix("PREVIEWLED:") {
        let mut parts = rest.split(':');
        match (parts.next(), parts.next()) {
            (Some(led), Some(color)) if !led.is_empty() && !color.is_empty() => {
                Command::PreviewLed { led, color }
            }
            _ => Command::Unknown,
        }
    } else if let Some(rest) = trimmed.strip_prefix("SETLED:") {
        let mut parts = rest.split(':');
        let first = parts.next();
        let second = parts.next();
        let third = parts.next();
        let fourth = parts.next();

        match (first, second, third, fourth) {
            (Some(index), Some(r), Some(g), Some(b)) => {
                let parsed = (
                    index.parse::<u8>(),
                    r.parse::<u8>(),
                    g.parse::<u8>(),
                    b.parse::<u8>(),
                );
                match parsed {
                    (Ok(index), Ok(r), Ok(g), Ok(b)) => Command::SetLedRgb { index, r, g, b },
                    _ => Command::Unknown,
                }
            }
            (Some(led), Some(color), None, None) if !led.is_empty() && !color.is_empty() => {
                Command::SetLedHex { led, color }
            }
            _ => Command::Unknown,
        }
    } else if trimmed == "LEDRESTORE" {
        Command::LedRestore
    } else if trimmed == "TILTWAVE" {
        Command::TiltWave
    } else if let Some(v) = trimmed.strip_prefix("TILTWAVE_ENABLE:") {
        Command::TiltWaveEnable {
            enabled: parse_bool_token(v),
        }
    } else if let Some(path) = trimmed.strip_prefix("MKDIR:") {
        Command::Mkdir { path }
    } else if trimmed == "REBOOT" {
        Command::Reboot
    } else if trimmed == "REBOOTBOOTSEL" {
        Command::RebootBootsel
    } else if trimmed == "END" {
        Command::End
    } else {
        Command::Unknown
    }
}

pub fn requires_ack(cmd: &Command<'_>) -> bool {
    matches!(
        cmd,
        Command::Ready
            | Command::ReadVersion
            | Command::ReadDeviceName
            | Command::ReadUid
            | Command::ReadFile { .. }
            | Command::ReadPin { .. }
            | Command::PreviewLed { .. }
            | Command::ReadWhammy
            | Command::ReadJoystick
            | Command::SetLedHex { .. }
            | Command::SetLedRgb { .. }
            | Command::LedRestore
            | Command::TiltWave
            | Command::TiltWaveEnable { .. }
            | Command::Demo
            | Command::DetectPin { .. }
            | Command::SavePin { .. }
            | Command::CancelPinDetect
            | Command::Reboot
            | Command::RebootBootsel
            | Command::Mkdir { .. }
    )
}

fn parse_bool_token(v: &str) -> bool {
    let t = v.trim();
    t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes") || t.eq_ignore_ascii_case("on")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParserMode {
    Idle,
    Write,
    WriteStream,
    MergeUser,
}

impl Copy for ParserMode {}

fn push_bytes(dst: &mut [u8], offset: usize, src: &[u8]) -> usize {
    if offset >= dst.len() {
        return 0;
    }
    let cap = dst.len() - offset;
    let n = core::cmp::min(cap, src.len());
    dst[offset..offset + n].copy_from_slice(&src[..n]);
    n
}

fn contains_ascii_nocase(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }

    let last_start = haystack.len() - needle.len();
    let mut i = 0;
    while i <= last_start {
        if haystack[i..i + needle.len()].eq_ignore_ascii_case(needle) {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        ack_head_20, filename_from_path, parse_command, requires_ack, select_write_mode, Command,
        OutboundMessage, ParserMode, ProtocolAction, ProtocolEngine, WriteMode,
    };

    #[test]
    fn parses_ready() {
        assert_eq!(parse_command("READY?"), Command::Ready);
    }

    #[test]
    fn parses_readfile() {
        assert_eq!(
            parse_command("READFILE:/config.json"),
            Command::ReadFile {
                path: "/config.json"
            }
        );
    }

    #[test]
    fn parses_writefile() {
        assert_eq!(
            parse_command("WRITEFILE:/config.json"),
            Command::WriteFile {
                path: "/config.json"
            }
        );
    }

    #[test]
    fn parses_savepin() {
        assert_eq!(
            parse_command("SAVEPIN:GREEN_FRET:GP1"),
            Command::SavePin {
                name: "GREEN_FRET",
                pin: "GP1"
            }
        );
    }

    #[test]
    fn parses_setled_rgb() {
        assert_eq!(
            parse_command("SETLED:2:255:128:0"),
            Command::SetLedRgb {
                index: 2,
                r: 255,
                g: 128,
                b: 0
            }
        );
    }

    #[test]
    fn parses_setled_hex() {
        assert_eq!(
            parse_command("SETLED:strum-up:#00FF00"),
            Command::SetLedHex {
                led: "strum-up",
                color: "#00FF00"
            }
        );
    }

    #[test]
    fn parses_tiltwave_enable_true() {
        assert_eq!(
            parse_command("TILTWAVE_ENABLE:on"),
            Command::TiltWaveEnable { enabled: true }
        );
    }

    #[test]
    fn parses_tiltwave_enable_false() {
        assert_eq!(
            parse_command("TILTWAVE_ENABLE:off"),
            Command::TiltWaveEnable { enabled: false }
        );
    }

    #[test]
    fn parses_read_commands() {
        assert_eq!(parse_command("READVERSION"), Command::ReadVersion);
        assert_eq!(parse_command("READDEVICENAME"), Command::ReadDeviceName);
        assert_eq!(parse_command("READUID"), Command::ReadUid);
        assert_eq!(parse_command("READWHAMMY"), Command::ReadWhammy);
        assert_eq!(parse_command("READJOYSTICK"), Command::ReadJoystick);
    }

    #[test]
    fn parses_pin_commands() {
        assert_eq!(
            parse_command("DETECTPIN:ORANGE_FRET"),
            Command::DetectPin {
                name: "ORANGE_FRET"
            }
        );
        assert_eq!(parse_command("CANCELPINDETECT"), Command::CancelPinDetect);
        assert_eq!(
            parse_command("READPIN:GREEN_FRET"),
            Command::ReadPin {
                name: "GREEN_FRET"
            }
        );
    }

    #[test]
    fn parses_misc_commands() {
        assert_eq!(parse_command("IMPORTUSER"), Command::ImportUser);
        assert_eq!(parse_command("DEMO"), Command::Demo);
        assert_eq!(parse_command("TILTWAVE"), Command::TiltWave);
        assert_eq!(parse_command("LEDRESTORE"), Command::LedRestore);
        assert_eq!(
            parse_command("MKDIR:/updates"),
            Command::Mkdir { path: "/updates" }
        );
        assert_eq!(parse_command("REBOOT"), Command::Reboot);
        assert_eq!(parse_command("REBOOTBOOTSEL"), Command::RebootBootsel);
        assert_eq!(parse_command("END"), Command::End);
    }

    #[test]
    fn ack_matrix_matches_legacy_expectations() {
        assert!(requires_ack(&parse_command("READY?")));
        assert!(requires_ack(&parse_command("READFILE:/config.json")));
        assert!(requires_ack(&parse_command("DETECTPIN:GREEN_FRET")));
        assert!(requires_ack(&parse_command("MKDIR:/updates")));
        assert!(!requires_ack(&parse_command("WRITEFILE:/config.json")));
        assert!(!requires_ack(&parse_command("IMPORTUSER")));
        assert!(!requires_ack(&parse_command("END")));
    }

    #[test]
    fn write_mode_selection_matches_legacy_streaming_heuristic() {
        assert_eq!(select_write_mode("/code.py"), WriteMode::Streaming);
        assert_eq!(select_write_mode("/config.json"), WriteMode::Buffered);
        assert_eq!(
            select_write_mode("/folder/something_long_name.py"),
            WriteMode::Streaming
        );
    }

    #[test]
    fn protocol_engine_transitions_write_mode() {
        let mut engine = ProtocolEngine::new();
        let o1 = engine.process_line("WRITEFILE:/config.json");
        assert_eq!(engine.mode(), ParserMode::Write);
        assert!(matches!(
            o1.action,
            ProtocolAction::Command(Command::WriteFile { path: "/config.json" })
        ));

        let o2 = engine.process_line("{\"a\":1}");
        assert!(matches!(o2.action, ProtocolAction::WriteChunk { line: "{\"a\":1}" }));

        let o3 = engine.process_line("END");
        assert!(matches!(o3.action, ProtocolAction::WriteCommit));
        assert_eq!(engine.mode(), ParserMode::Idle);
    }

    #[test]
    fn protocol_engine_transitions_merge_mode() {
        let mut engine = ProtocolEngine::new();
        let o1 = engine.process_line("IMPORTUSER");
        assert_eq!(engine.mode(), ParserMode::MergeUser);
        assert!(matches!(o1.action, ProtocolAction::Command(Command::ImportUser)));

        let o2 = engine.process_line("{}\n");
        assert!(matches!(o2.action, ProtocolAction::MergeChunk { line: "{}" }));

        let o3 = engine.process_line("END");
        assert!(matches!(o3.action, ProtocolAction::MergeCommit));
        assert_eq!(engine.mode(), ParserMode::Idle);
    }

    #[test]
    fn ack_head_is_trimmed_to_legacy_length() {
        assert_eq!(ack_head_20("READY?"), "READY?");
        assert_eq!(
            ack_head_20("READFILE:/a/very/long/path/config.json"),
            "READFILE:/a/very/lon"
        );
    }

    #[test]
    fn filename_extraction_works_for_protocol_markers() {
        assert_eq!(filename_from_path("/config.json"), "config.json");
        assert_eq!(filename_from_path("config.json"), "config.json");
    }

    #[test]
    fn outbound_render_matches_legacy_envelopes() {
        let mut b = [0u8; 128];

        let n = OutboundMessage::WriteFileReady {
            filename: "config.json",
        }
        .write_into(&mut b);
        let s = core::str::from_utf8(&b[..n]).unwrap_or("");
        assert_eq!(s, "WRITEFILE:READY:config.json\n");

        let n = OutboundMessage::StreamReady {
            filename: "code.py",
        }
        .write_into(&mut b);
        let s = core::str::from_utf8(&b[..n]).unwrap_or("");
        assert_eq!(s, "STREAM:READY:code.py\n");

        let n = OutboundMessage::StartFile {
            filename: "config.json",
        }
        .write_into(&mut b);
        let s = core::str::from_utf8(&b[..n]).unwrap_or("");
        assert_eq!(s, "START_config.json\n");

        let n = OutboundMessage::EndFile {
            filename: "config.json",
        }
        .write_into(&mut b);
        let s = core::str::from_utf8(&b[..n]).unwrap_or("");
        assert_eq!(s, "END_config.json\n");
    }
}
