#![no_std]

use firmware_core::{default_hat_from_dpad, InputState, PackedReport, PlatformProfile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetProfile {
    PcParity,
    XboxOneExperimental,
    XboxSeriesExperimental,
    Xbox360Experimental,
    Ps3Experimental,
    Ps4Experimental,
}

pub struct PcParityProfile;

impl PlatformProfile for PcParityProfile {
    fn profile_name(&self) -> &'static str {
        "pc-parity"
    }

    fn pack_report(&self, state: &InputState) -> PackedReport {
        // Match legacy 4-byte style behavior first, then evolve per-profile.
        let mut buttons = 0u16;
        set_button(&mut buttons, 0, state.green);
        set_button(&mut buttons, 1, state.red);
        set_button(&mut buttons, 2, state.yellow);
        set_button(&mut buttons, 3, state.blue);
        set_button(&mut buttons, 4, state.orange);
        set_button(&mut buttons, 5, state.strum_up);
        set_button(&mut buttons, 6, state.strum_down);
        set_button(&mut buttons, 7, state.select);
        set_button(&mut buttons, 8, state.start);
        set_button(&mut buttons, 9, state.tilt);
        set_button(&mut buttons, 10, state.guide);

        let hat = default_hat_from_dpad(state) & 0x0f;

        let mut out = PackedReport::empty();
        out.bytes[0] = (buttons & 0xff) as u8;
        out.bytes[1] = ((buttons >> 8) & 0x07) as u8;
        out.bytes[2] = hat | 0xf0;
        out.bytes[3] = state.whammy;
        out.len = 4;
        out
    }
}

fn set_button(bitmap: &mut u16, index: u8, pressed: bool) {
    if pressed {
        *bitmap |= 1u16 << index;
    }
}
