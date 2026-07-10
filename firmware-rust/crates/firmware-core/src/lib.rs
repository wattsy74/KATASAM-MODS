#![no_std]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputState {
    pub green: bool,
    pub red: bool,
    pub yellow: bool,
    pub blue: bool,
    pub orange: bool,
    pub strum_up: bool,
    pub strum_down: bool,
    pub select: bool,
    pub start: bool,
    pub tilt: bool,
    pub guide: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub whammy: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedReport {
    pub bytes: [u8; 16],
    pub len: usize,
}

impl PackedReport {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; 16],
            len: 0,
        }
    }
}

pub trait PlatformProfile {
    fn profile_name(&self) -> &'static str;
    fn pack_report(&self, state: &InputState) -> PackedReport;
}

pub fn default_hat_from_dpad(state: &InputState) -> u8 {
    let up = state.dpad_up;
    let down = state.dpad_down;
    let left = state.dpad_left;
    let right = state.dpad_right;

    if up && right {
        1
    } else if down && right {
        3
    } else if down && left {
        5
    } else if up && left {
        7
    } else if up {
        0
    } else if right {
        2
    } else if down {
        4
    } else if left {
        6
    } else {
        0x0f
    }
}
