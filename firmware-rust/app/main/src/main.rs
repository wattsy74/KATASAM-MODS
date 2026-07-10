use config_protocol::{parse_command, Command};
use config_runtime::Runtime;
use firmware_core::{InputState, PlatformProfile};
use platform_profiles::PcParityProfile;
use std::io::{self, BufRead, Write};

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
    let mut runtime = Runtime::new();

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

fn print_help() {
    println!("katasam-firmware-main options:");
    println!("  --protocol-stdio  run line-based protocol bridge over stdin/stdout");
    println!("  --help, -h        show this help");
    println!("  (no args)         run local demo/check output");
}
