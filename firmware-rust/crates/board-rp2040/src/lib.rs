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
