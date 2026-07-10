use config_protocol::{parse_command, Command};
use config_runtime::Runtime;
use firmware_core::{InputState, PlatformProfile};
use platform_profiles::PcParityProfile;
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--protocol-stdio") {
        if let Err(e) = run_protocol_stdio() {
            eprintln!("protocol-stdio error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    run_demo();
}

fn run_demo() {
    // Host-side sanity check while embedded runtime is brought up.
    let profile = PcParityProfile;
    let input = InputState {
        green: true,
        red: false,
        yellow: false,
        blue: false,
        orange: false,
        strum_up: false,
        strum_down: true,
        select: false,
        start: false,
        tilt: false,
        guide: false,
        dpad_up: false,
        dpad_down: false,
        dpad_left: false,
        dpad_right: false,
        whammy: 127,
    };

    let report = profile.pack_report(&input);
    println!(
        "profile={}, len={}, bytes={:?}",
        profile.profile_name(),
        report.len,
        &report.bytes[..report.len]
    );

    match parse_command("READFILE:/config.json") {
        Command::ReadFile { path } => println!("protocol ok: {}", path),
        _ => println!("protocol parse mismatch"),
    }

    println!("demo complete. use --protocol-stdio for line-based protocol bridge");
}

fn run_protocol_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut runtime = seed_runtime_from_repo_files();

    for line_result in stdin.lock().lines() {
        let line = line_result?;
        let responses = runtime.process_line(&line);
        for response in responses {
            stdout.write_all(response.as_bytes())?;
        }
        stdout.flush()?;
    }

    Ok(())
}

fn seed_runtime_from_repo_files() -> Runtime {
    let mut runtime = Runtime::new();
    let root = repo_root_from_cwd();

    let config_path = root.join("firmware/config.json");
    if let Ok(content) = fs::read_to_string(&config_path) {
        let normalized = normalize_seed_file(content);
        runtime = runtime.with_file("/config.json", &normalized);
    }

    let presets_path = root.join("firmware/user_presets.json");
    if let Ok(content) = fs::read_to_string(&presets_path) {
        let normalized = normalize_seed_file(content);
        runtime = runtime.with_file("/user_presets.json", &normalized);
    }

    if let Ok(path) = std::env::var("KATASAM_CONFIG_OVERRIDE_FILE") {
        if let Ok(content) = fs::read_to_string(path) {
            runtime = runtime.with_file("/config.json", &content);
        }
    }

    if let Ok(path) = std::env::var("KATASAM_USER_PRESETS_OVERRIDE_FILE") {
        if let Ok(content) = fs::read_to_string(path) {
            runtime = runtime.with_file("/user_presets.json", &content);
        }
    }

    let version = std::env::var("KATASAM_RUNTIME_VERSION").unwrap_or_default();
    let device_name = std::env::var("KATASAM_DEVICE_NAME").unwrap_or_default();
    let uid = std::env::var("KATASAM_DEVICE_UID").unwrap_or_default();
    runtime = runtime.with_identity(&version, &device_name, &uid);

    let whammy = parse_i32_env("KATASAM_WHAMMY_VALUE", -1);
    let joystick_x = parse_i32_env("KATASAM_JOYSTICK_X", -1);
    let joystick_y = parse_i32_env("KATASAM_JOYSTICK_Y", -1);
    runtime = runtime.with_controls(whammy, joystick_x, joystick_y);

    if let Ok(pin_map) = std::env::var("KATASAM_PIN_READS") {
        for entry in pin_map.split(';') {
            let e = entry.trim();
            if e.is_empty() {
                continue;
            }
            if let Some((name, value)) = e.split_once('=') {
                runtime = runtime.with_pin_read(name, value);
            }
        }
    }

    runtime
}

fn parse_i32_env(key: &str, default: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(default)
}

fn repo_root_from_cwd() -> PathBuf {
    // Expected invocation is from firmware-rust, but support being run from nested dirs.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("firmware").is_dir() {
        return cwd;
    }
    if cwd.file_name().and_then(|n| n.to_str()) == Some("firmware-rust") {
        return cwd
            .parent()
            .map_or_else(|| cwd.clone(), std::path::Path::to_path_buf);
    }
    cwd
}

fn ensure_trailing_newline(mut content: String) -> String {
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

fn normalize_seed_file(content: String) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(&content) {
        return ensure_trailing_newline(value.to_string());
    }
    ensure_trailing_newline(content)
}

fn print_help() {
    println!("katasam-firmware-main options:");
    println!("  --protocol-stdio  run line-based protocol bridge over stdin/stdout");
    println!("  --help, -h        show this help");
    println!("  (no args)         run local demo/check output");
}
