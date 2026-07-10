#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use board_rp2040::is_bootsel_command;
#[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
use embedded_hal::adc::OneShot;
#[cfg(not(feature = "rescue-mode"))]
use embedded_hal::digital::v2::InputPin;
use rp2040_hal as hal;
#[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
use rp2040_hal::adc::{Adc, AdcPin};
use rp2040_hal::clocks::init_clocks_and_plls;
#[cfg(not(feature = "rescue-mode"))]
use rp2040_hal::sio::Sio;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::watchdog::Watchdog;
use usb_device::bus::UsbBusAllocator;
use usb_device::class_prelude::UsbClass;
use usb_device::device::StringDescriptors;
use usb_device::prelude::*;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;

#[cfg(all(feature = "boot2-w25q080", feature = "boot2-is25lp080"))]
compile_error!("Choose only one boot2 feature");
#[cfg(all(feature = "boot2-w25q080", feature = "boot2-generic-03h"))]
compile_error!("Choose only one boot2 feature");
#[cfg(all(feature = "boot2-is25lp080", feature = "boot2-generic-03h"))]
compile_error!("Choose only one boot2 feature");
#[cfg(not(any(
    feature = "boot2-w25q080",
    feature = "boot2-is25lp080",
    feature = "boot2-generic-03h"
)))]
compile_error!("One boot2 feature must be selected");

const USB_VID: u16 = 0x2E8A;
const USB_PID: u16 = 0x1031;
const REPORT_INTERVAL_US: u64 = 8_000;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (usage_page = BUTTON, usage_min = 1, usage_max = 8) = {
            #[packed_bits 8] buttons_lo=input;
        };
        (usage_page = BUTTON, usage_min = 9, usage_max = 16) = {
            #[packed_bits 8] buttons_hi=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = X) = {
            #[item_settings data,variable,absolute] x=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = Y) = {
            #[item_settings data,variable,absolute] y=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = Z) = {
            #[item_settings data,variable,absolute] z=input;
        };
        (usage_page = GENERIC_DESKTOP, usage = WHEEL) = {
            #[item_settings data,variable,absolute] wheel=input;
        };
    }
)]
pub struct GamepadReport {
    pub buttons_lo: u8,
    pub buttons_hi: u8,
    pub x: i8,
    pub y: i8,
    pub z: u8,
    pub wheel: i8,
}

#[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
fn axis_from_adc_u8(raw: u16) -> u8 {
    // RP2040 ADC is 12-bit (0..4095). Map to unsigned HID axis (0..255).
    ((raw as u32 * 255) / 4095) as u8
}

#[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
fn axis_from_adc_i8(raw: u16) -> i8 {
    // Mirror the same source on a signed axis for host/game compatibility.
    let centered = ((raw as i32 * 255) / 4095) - 127;
    centered.clamp(-127, 127) as i8
}

#[cfg(not(feature = "rescue-mode"))]
fn axis_from_dpad(negative_pressed: bool, positive_pressed: bool) -> i8 {
    match (negative_pressed, positive_pressed) {
        (true, false) => -127,
        (false, true) => 127,
        _ => 0,
    }
}

fn maybe_enter_bootsel(line: &[u8]) {
    if is_bootsel_command(line) {
        hal::rom_data::reset_to_usb_boot(0, 0);
    }
}

#[link_section = ".boot2"]
#[used]
#[cfg(feature = "boot2-w25q080")]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[link_section = ".boot2"]
#[used]
#[cfg(feature = "boot2-is25lp080")]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_IS25LP080;

#[link_section = ".boot2"]
#[used]
#[cfg(feature = "boot2-generic-03h")]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    #[cfg(not(feature = "rescue-mode"))]
    let sio = Sio::new(pac.SIO);

    #[cfg(not(feature = "rescue-mode"))]
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // V2 mapping from hardware table.
    #[cfg(not(feature = "rescue-mode"))]
    let green_fret = pins.gpio2.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let red_fret = pins.gpio3.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let yellow_fret = pins.gpio4.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let blue_fret = pins.gpio5.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let orange_fret = pins.gpio6.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let guide = pins.gpio7.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let dpad_right = pins.gpio8.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let dpad_left = pins.gpio9.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let dpad_down = pins.gpio10.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let dpad_up = pins.gpio11.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let tilt = pins.gpio12.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let strum_down = pins.gpio14.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let strum_up = pins.gpio15.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let select = pins.gpio1.into_pull_up_input();
    #[cfg(not(feature = "rescue-mode"))]
    let start = pins.gpio0.into_pull_up_input();

    #[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
    let mut whammy_adc = AdcPin::new(pins.gpio29.into_floating_input()).ok();

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

    #[cfg(all(not(feature = "rescue-mode"), feature = "v2-analog"))]
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);

    let mut serial_buf = [0u8; 64];
    let mut line_buf = [0u8; 64];
    let mut line_len = 0usize;
    let mut next_report_at = timer.get_counter().ticks();
    #[cfg(feature = "rescue-mode")]
    let mut phase: u8 = 0;

    loop {
        let mut classes: [&mut dyn UsbClass<UsbBus>; 2] = [&mut serial, &mut hid];
        if usb_dev.poll(&mut classes) {
            match serial.read(&mut serial_buf) {
                Ok(count) if count > 0 => {
                    for byte in &serial_buf[..count] {
                        if *byte == b'\n' || *byte == b'\r' {
                            if line_len > 0 {
                                let line = &line_buf[..line_len];
                                maybe_enter_bootsel(line);

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
            #[cfg(feature = "rescue-mode")]
            {
                phase = phase.wrapping_add(1);
                let report = GamepadReport {
                    buttons_lo: if (phase & 0x20) == 0 { 0x01 } else { 0x02 },
                    buttons_hi: 0,
                    x: ((phase as i16) - 128) as i8,
                    y: (127 - phase as i16) as i8,
                    z: 0,
                    wheel: 0,
                };
                let _ = hid.push_input(&report);
                next_report_at = now + REPORT_INTERVAL_US;
                continue;
            }

            #[cfg(not(feature = "rescue-mode"))]
            {
            let green_pressed = green_fret.is_low().unwrap_or(false);
            let red_pressed = red_fret.is_low().unwrap_or(false);
            let yellow_pressed = yellow_fret.is_low().unwrap_or(false);
            let blue_pressed = blue_fret.is_low().unwrap_or(false);
            let orange_pressed = orange_fret.is_low().unwrap_or(false);
            let strum_up_pressed = strum_up.is_low().unwrap_or(false);
            let strum_down_pressed = strum_down.is_low().unwrap_or(false);
            let start_pressed = start.is_low().unwrap_or(false);
            let select_pressed = select.is_low().unwrap_or(false);
            let guide_pressed = guide.is_low().unwrap_or(false);
            let tilt_pressed = tilt.is_low().unwrap_or(false);

            let dpad_left_pressed = dpad_left.is_low().unwrap_or(false);
            let dpad_right_pressed = dpad_right.is_low().unwrap_or(false);
            let dpad_up_pressed = dpad_up.is_low().unwrap_or(false);
            let dpad_down_pressed = dpad_down.is_low().unwrap_or(false);

            let mut buttons_lo: u8 = 0;
            if green_pressed {
                buttons_lo |= 1 << 0;
            }
            if red_pressed {
                buttons_lo |= 1 << 1;
            }
            if yellow_pressed {
                buttons_lo |= 1 << 2;
            }
            if blue_pressed {
                buttons_lo |= 1 << 3;
            }
            if orange_pressed {
                buttons_lo |= 1 << 4;
            }
            if strum_up_pressed {
                buttons_lo |= 1 << 5;
            }
            if strum_down_pressed {
                buttons_lo |= 1 << 6;
            }
            if start_pressed {
                buttons_lo |= 1 << 7;
            }

            let mut buttons_hi: u8 = 0;
            if select_pressed {
                buttons_hi |= 1 << 0; // button 9
            }
            if guide_pressed {
                buttons_hi |= 1 << 1; // button 10
            }
            if tilt_pressed {
                buttons_hi |= 1 << 2; // button 11
            }
            if dpad_up_pressed {
                buttons_hi |= 1 << 3; // button 12
            }
            if dpad_down_pressed {
                buttons_hi |= 1 << 4; // button 13
            }
            if dpad_left_pressed {
                buttons_hi |= 1 << 5; // button 14
            }
            if dpad_right_pressed {
                buttons_hi |= 1 << 6; // button 15
            }

            let y_axis = axis_from_dpad(dpad_up_pressed, dpad_down_pressed);

            #[cfg(feature = "v2-analog")]
            let mut z_axis: u8 = 0;

            #[cfg(feature = "v2-analog")]
            let mut wheel_axis: i8 = 0;

            #[cfg(feature = "v2-analog")]
            {
                if let Some(ref mut whammy_channel) = whammy_adc {
                    if let Ok(whammy_raw) = adc.read(whammy_channel) {
                        z_axis = axis_from_adc_u8(whammy_raw);
                        wheel_axis = axis_from_adc_i8(whammy_raw);
                    }
                }
            }

            #[cfg(not(feature = "v2-analog"))]
            let z_axis: u8 = 0;

            #[cfg(not(feature = "v2-analog"))]
            let wheel_axis: i8 = 0;

            let report = GamepadReport {
                buttons_lo,
                buttons_hi,
                x: axis_from_dpad(dpad_left_pressed, dpad_right_pressed),
                y: y_axis,
                z: z_axis,
                wheel: wheel_axis,
            };

            let _ = hid.push_input(&report);
            next_report_at = now + REPORT_INTERVAL_US;
            }
        }
    }
}
