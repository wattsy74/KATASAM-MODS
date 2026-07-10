#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use board_rp2040::is_bootsel_command;
use rp2040_hal as hal;
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::watchdog::Watchdog;
use usb_device::bus::UsbBusAllocator;
use usb_device::class_prelude::UsbClass;
use usb_device::device::StringDescriptors;
use usb_device::prelude::*;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;

const USB_VID: u16 = 0x2E8A;
const USB_PID: u16 = 0x1031;
const REPORT_INTERVAL_US: u64 = 8_000;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (usage_page = BUTTON, usage_min = 1, usage_max = 8) = {
            #[packed_bits 8] buttons=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = X) = {
            #[item_settings data,variable,absolute] x=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = Y) = {
            #[item_settings data,variable,absolute] y=input;
        };
    }
)]
pub struct GamepadReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
}

fn maybe_enter_bootsel(line: &[u8]) {
    if is_bootsel_command(line) {
        hal::rom_data::reset_to_usb_boot(0, 0);
    }
}

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        12_000_000,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);
    let mut hid = HIDClass::new(&usb_bus, GamepadReport::desc(), 10);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(USB_VID, USB_PID))
        .strings(&[StringDescriptors::default()
            .manufacturer("KATASAM")
            .product("KATASAM Rust Test FW")
            .serial_number("TEST-0001")])
        .unwrap()
        .device_class(0x00)
        .build();

    let mut serial_buf = [0u8; 64];
    let mut line_buf = [0u8; 64];
    let mut line_len = 0usize;
    let mut phase: u8 = 0;
    let mut next_report_at = timer.get_counter().ticks();

    loop {
        let mut classes: [&mut dyn UsbClass<UsbBus>; 2] = [&mut serial, &mut hid];
        if usb_dev.poll(&mut classes) {
            match serial.read(&mut serial_buf) {
                Ok(count) if count > 0 => {
                    for byte in &serial_buf[..count] {
                        if *byte == b'\n' || *byte == b'\r' {
                            if line_len > 0 {
                                maybe_enter_bootsel(&line_buf[..line_len]);
                                line_len = 0;
                            }
                            continue;
                        }

                        if line_len < line_buf.len() {
                            line_buf[line_len] = *byte;
                            line_len += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        let now = timer.get_counter().ticks();
        if now >= next_report_at {
            phase = phase.wrapping_add(1);

            // Stream changing axes/buttons so host-side gamepad visualization updates continuously.
            let report = GamepadReport {
                buttons: if (phase & 0x20) == 0 { 0x01 } else { 0x02 },
                x: ((phase as i16) - 128) as i8,
                y: (127 - phase as i16) as i8,
            };
            let _ = hid.push_input(&report);
            next_report_at = now + REPORT_INTERVAL_US;
        }
    }
}
