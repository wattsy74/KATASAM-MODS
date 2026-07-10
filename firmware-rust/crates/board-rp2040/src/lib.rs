#![no_std]

use firmware_core::InputState;

pub trait BoardSupport {
    fn read_inputs(&mut self) -> InputState;
    fn write_report(&mut self, report: &[u8]);
    fn reboot_normal(&mut self);
    fn reboot_bootsel(&mut self);
}

pub struct Rp2040Board;

impl Rp2040Board {
    pub const fn new() -> Self {
        Self
    }
}

pub const BOOTSEL_COMMAND: &[u8] = b"KATASAM_BOOTSEL_V1";
pub const REBOOT_BOOTSEL_COMMAND: &[u8] = b"KATASAM_REBOOT_BOOTSEL_V1";

pub fn is_bootsel_command(line: &[u8]) -> bool {
    let trimmed = trim_ascii(line);
    trimmed.eq_ignore_ascii_case(BOOTSEL_COMMAND)
        || trimmed.eq_ignore_ascii_case(REBOOT_BOOTSEL_COMMAND)
}

fn trim_ascii(input: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = input.len();

    while start < end && matches!(input[start], b'\r' | b'\n' | b' ' | b'\t') {
        start += 1;
    }

    while end > start && matches!(input[end - 1], b'\r' | b'\n' | b' ' | b'\t') {
        end -= 1;
    }

    &input[start..end]
}

#[cfg(test)]
mod tests {
    use super::is_bootsel_command;

    #[test]
    fn matches_bootsel_variants() {
        assert!(is_bootsel_command(b"KATASAM_BOOTSEL_V1"));
        assert!(is_bootsel_command(b"katasam_bootsel_v1\n"));
        assert!(is_bootsel_command(b"  KATASAM_REBOOT_BOOTSEL_V1\r\n"));
        assert!(!is_bootsel_command(b"BOOTSEL"));
        assert!(!is_bootsel_command(b"noop"));
    }
}
