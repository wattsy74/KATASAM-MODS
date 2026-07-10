#![no_std]
#![no_main]

use cortex_m_rt::entry;
use heapless::Vec as HeapVec;
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
use rp2040_hal::clocks::Clock;
#[cfg(not(feature = "rescue-mode"))]
use rp2040_hal::pio::PIOExt;
#[cfg(not(feature = "rescue-mode"))]
use rp2040_hal::sio::Sio;
use rp2040_hal::usb::UsbBus;
use rp2040_hal::watchdog::Watchdog;
#[cfg(not(feature = "rescue-mode"))]
use smart_leds_trait::{RGB8, SmartLedsWrite};
use usb_device::bus::UsbBusAllocator;
use usb_device::class_prelude::UsbClass;
use usb_device::device::StringDescriptors;
use usb_device::prelude::*;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;
#[cfg(not(feature = "rescue-mode"))]
use ws2812_pio::Ws2812;
use cortex_m::peripheral::SCB;

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
const FW_VERSION: &str = "1.0.0";
const DEVICE_NAME: &str = "KATASAM Rust Test FW";
const DEVICE_UID: &str = "A1B2C3D4E5F60708";

const DEFAULT_CONFIG_JSON: &str = include_str!("../../../firmware/config.json");
const DEFAULT_PRESETS_JSON: &str = include_str!("../../../firmware/presets.json");
const DEFAULT_USER_PRESETS_JSON: &str = include_str!("../../../firmware/user_presets.json");

const RX_LINE_CAP: usize = 256;
const WRITE_LINE_CAP: usize = WRITE_BUF_CAP;
const WRITE_BUF_CAP: usize = 16384;
const TX_BUF_CAP: usize = 24576;
const CONFIG_BUF_CAP: usize = 8192;
const PRESETS_BUF_CAP: usize = 12288;
const USER_PRESETS_BUF_CAP: usize = 16384;
const FLASH_STORAGE_BASE_OFFSET: usize = 0x000F_4000;
const FLASH_STORAGE_REGION_SIZE: usize = 0x0000_4000;
const FLASH_ERASE_SIZE: usize = 4096;
const FLASH_PROGRAM_SIZE: usize = 256;
const FLASH_ERASE_CMD_SECTOR: u8 = 0x20;
const STORAGE_HEADER_SIZE: usize = 16;

const MAGIC_CONFIG: [u8; 4] = *b"KCFG";
const MAGIC_PRESETS: [u8; 4] = *b"KPRE";
const MAGIC_USER_PRESETS: [u8; 4] = *b"KUSR";
#[cfg(not(feature = "rescue-mode"))]
const LED_COUNT: usize = 7;
#[cfg(not(feature = "rescue-mode"))]
const TILTWAVE_MAX_STEPS: u16 = 120;
#[cfg(not(feature = "rescue-mode"))]
const TILTWAVE_CYCLES: u16 = 3;

#[cfg(not(feature = "rescue-mode"))]
const LED_PRESSED: [RGB8; LED_COUNT] = [
    RGB8 { r: 255, g: 255, b: 255 },
    RGB8 { r: 255, g: 255, b: 255 },
    RGB8 { r: 179, g: 62, b: 0 },
    RGB8 { r: 0, g: 0, b: 255 },
    RGB8 { r: 255, g: 255, b: 0 },
    RGB8 { r: 255, g: 0, b: 0 },
    RGB8 { r: 0, g: 255, b: 0 },
];

#[cfg(not(feature = "rescue-mode"))]
const LED_RELEASED: [RGB8; LED_COUNT] = [
    RGB8 { r: 69, g: 69, b: 69 },
    RGB8 { r: 69, g: 69, b: 69 },
    RGB8 { r: 82, g: 28, b: 0 },
    RGB8 { r: 0, g: 0, b: 145 },
    RGB8 { r: 105, g: 107, b: 0 },
    RGB8 { r: 140, g: 0, b: 9 },
    RGB8 { r: 0, g: 61, b: 0 },
];

#[cfg(not(feature = "rescue-mode"))]
const TILTWAVE_COLORS: [RGB8; 19] = [
    RGB8 { r: 0, g: 0, b: 255 },
    RGB8 { r: 0, g: 100, b: 255 },
    RGB8 { r: 0, g: 150, b: 255 },
    RGB8 { r: 50, g: 200, b: 255 },
    RGB8 { r: 100, g: 220, b: 255 },
    RGB8 { r: 150, g: 240, b: 255 },
    RGB8 { r: 200, g: 250, b: 255 },
    RGB8 { r: 255, g: 255, b: 255 },
    RGB8 { r: 200, g: 250, b: 255 },
    RGB8 { r: 150, g: 240, b: 255 },
    RGB8 { r: 100, g: 220, b: 255 },
    RGB8 { r: 50, g: 200, b: 255 },
    RGB8 { r: 0, g: 150, b: 255 },
    RGB8 { r: 0, g: 100, b: 255 },
    RGB8 { r: 0, g: 50, b: 255 },
    RGB8 { r: 0, g: 25, b: 128 },
    RGB8 { r: 0, g: 12, b: 64 },
    RGB8 { r: 0, g: 0, b: 32 },
    RGB8 { r: 0, g: 0, b: 0 },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriteTarget {
    Config,
    Presets,
    UserPresets,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SerialMode {
    Command,
    WriteFile(WriteTarget),
    ImportUser,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingFlashCommit {
    WriteFile(WriteTarget),
    ImportUser,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PersistError {
    Unsupported,
    Empty,
    TooLarge,
    RegionTooSmall,
}

fn starts_with_ignore_ascii_case(haystack: &str, prefix: &str) -> bool {
    haystack
        .as_bytes()
        .get(..prefix.len())
        .map(|head| head.eq_ignore_ascii_case(prefix.as_bytes()))
        .unwrap_or(false)
}

fn trim_ascii_spaces(input: &str) -> &str {
    let bytes = input.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();

    while start < end && matches!(bytes[start], b'\r' | b'\n' | b' ' | b'\t') {
        start += 1;
    }

    while end > start && matches!(bytes[end - 1], b'\r' | b'\n' | b' ' | b'\t') {
        end -= 1;
    }

    &input[start..end]
}

fn parse_bool_token(input: &str) -> Option<bool> {
    let token = trim_ascii_spaces(input);
    if token.eq_ignore_ascii_case("true")
        || token.eq_ignore_ascii_case("on")
        || token.eq_ignore_ascii_case("yes")
        || token == "1"
    {
        Some(true)
    } else if token.eq_ignore_ascii_case("false")
        || token.eq_ignore_ascii_case("off")
        || token.eq_ignore_ascii_case("no")
        || token == "0"
    {
        Some(false)
    } else {
        None
    }
}

fn command_requires_ack(line: &str) -> bool {
    starts_with_ignore_ascii_case(line, "FIRMWARE_READY?")
        || starts_with_ignore_ascii_case(line, "READY?")
        || starts_with_ignore_ascii_case(line, "READVERSION")
        || starts_with_ignore_ascii_case(line, "READDEVICENAME")
        || starts_with_ignore_ascii_case(line, "READUID")
        || starts_with_ignore_ascii_case(line, "READFILE:")
        || starts_with_ignore_ascii_case(line, "READPIN:")
        || starts_with_ignore_ascii_case(line, "READWHAMMY")
        || starts_with_ignore_ascii_case(line, "READJOYSTICK")
        || starts_with_ignore_ascii_case(line, "PREVIEWLED:")
        || starts_with_ignore_ascii_case(line, "SETLED:")
        || starts_with_ignore_ascii_case(line, "LEDRESTORE")
        || starts_with_ignore_ascii_case(line, "DEMO")
        || starts_with_ignore_ascii_case(line, "TILTWAVE")
        || starts_with_ignore_ascii_case(line, "TILTWAVE_ENABLE:")
        || starts_with_ignore_ascii_case(line, "DETECTPIN:")
        || starts_with_ignore_ascii_case(line, "SAVEPIN:")
        || starts_with_ignore_ascii_case(line, "CANCELPINDETECT")
        || starts_with_ignore_ascii_case(line, "REBOOT")
        || starts_with_ignore_ascii_case(line, "REBOOTBOOTSEL")
        || starts_with_ignore_ascii_case(line, "MKDIR:")
}

fn target_from_path(path: &str) -> WriteTarget {
    let normalized = trim_ascii_spaces(path).trim_start_matches('/');
    if normalized.eq_ignore_ascii_case("config.json") {
        WriteTarget::Config
    } else if normalized.eq_ignore_ascii_case("presets.json") {
        WriteTarget::Presets
    } else if normalized.eq_ignore_ascii_case("user_presets.json") {
        WriteTarget::UserPresets
    } else {
        WriteTarget::Unsupported
    }
}

fn filename_from_path(path: &str) -> &str {
    let p = trim_ascii_spaces(path).trim_matches('/');
    if p.is_empty() {
        return "unknown";
    }
    p.rsplit('/').next().unwrap_or("unknown")
}

fn tx_push_bytes<const N: usize>(tx: &mut HeapVec<u8, N>, bytes: &[u8]) {
    let free = N.saturating_sub(tx.len());
    let count = core::cmp::min(free, bytes.len());
    let _ = tx.extend_from_slice(&bytes[..count]);
}

fn tx_push_str<const N: usize>(tx: &mut HeapVec<u8, N>, s: &str) {
    tx_push_bytes(tx, s.as_bytes());
}

fn tx_push_u32<const N: usize>(tx: &mut HeapVec<u8, N>, mut value: u32) {
    if value == 0 {
        tx_push_str(tx, "0");
        return;
    }

    let mut digits = [0u8; 10];
    let mut idx = digits.len();
    while value > 0 {
        idx -= 1;
        digits[idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    tx_push_bytes(tx, &digits[idx..]);
}

fn tx_push_ack<const N: usize>(tx: &mut HeapVec<u8, N>, line: &str) {
    tx_push_str(tx, "ACK: ");
    let head = trim_ascii_spaces(line);
    let end = core::cmp::min(head.len(), 20);
    tx_push_bytes(tx, &head.as_bytes()[..end]);
    tx_push_str(tx, "\n");
}

fn tx_push_file<const N: usize>(
    tx: &mut HeapVec<u8, N>,
    filename: &str,
    content: &[u8],
) {
    tx_push_str(tx, "START_");
    tx_push_str(tx, filename);
    tx_push_str(tx, "\n");
    tx_push_bytes(tx, content);
    if content.last().copied() != Some(b'\n') {
        tx_push_str(tx, "\n");
    }
    tx_push_str(tx, "END_");
    tx_push_str(tx, filename);
    tx_push_str(tx, "\n");
}

fn flush_tx<const N: usize>(serial: &mut SerialPort<'_, UsbBus>, tx: &mut HeapVec<u8, N>) {
    if tx.is_empty() {
        return;
    }

    let chunk_len = core::cmp::min(64, tx.len());
    match serial.write(&tx[..chunk_len]) {
        Ok(written) if written > 0 => {
            let remaining = tx.len().saturating_sub(written);
            if remaining > 0 {
                tx.copy_within(written.., 0);
            }
            tx.truncate(remaining);
        }
        _ => {}
    }
}

fn copy_default<const N: usize>(dst: &mut HeapVec<u8, N>, text: &str) {
    dst.clear();
    let bytes = text.as_bytes();
    let copy_len = core::cmp::min(bytes.len(), N);
    let _ = dst.extend_from_slice(&bytes[..copy_len]);
}

fn storage_layout_for(target: WriteTarget) -> Option<(usize, usize, usize, [u8; 4])> {
    match target {
        WriteTarget::Config => Some((
            FLASH_STORAGE_BASE_OFFSET,
            FLASH_STORAGE_REGION_SIZE,
            CONFIG_BUF_CAP,
            MAGIC_CONFIG,
        )),
        WriteTarget::Presets => Some((
            FLASH_STORAGE_BASE_OFFSET + FLASH_STORAGE_REGION_SIZE,
            FLASH_STORAGE_REGION_SIZE,
            PRESETS_BUF_CAP,
            MAGIC_PRESETS,
        )),
        WriteTarget::UserPresets => Some((
            FLASH_STORAGE_BASE_OFFSET + (2 * FLASH_STORAGE_REGION_SIZE),
            FLASH_STORAGE_REGION_SIZE,
            USER_PRESETS_BUF_CAP,
            MAGIC_USER_PRESETS,
        )),
        WriteTarget::Unsupported => None,
    }
}

fn checksum32(data: &[u8]) -> u32 {
    let mut acc: u32 = 0x811C9DC5;
    for b in data {
        acc ^= *b as u32;
        acc = acc.wrapping_mul(16777619);
    }
    acc
}

fn flash_xip_read(offset: usize, len: usize) -> &'static [u8] {
    let ptr = (0x1000_0000usize + offset) as *const u8;
    unsafe { core::slice::from_raw_parts(ptr, len) }
}

fn load_blob_from_flash<const N: usize>(
    target: WriteTarget,
    dst: &mut HeapVec<u8, N>,
) -> bool {
    let Some((offset, region_size, max_payload, magic)) = storage_layout_for(target) else {
        return false;
    };

    let region = flash_xip_read(offset, region_size);
    if region.len() < STORAGE_HEADER_SIZE {
        return false;
    }

    if region[0..4] != magic {
        return false;
    }

    let payload_len = u32::from_le_bytes([region[4], region[5], region[6], region[7]]) as usize;
    let expected_checksum = u32::from_le_bytes([region[8], region[9], region[10], region[11]]);

    if payload_len == 0 || payload_len > max_payload {
        return false;
    }

    let payload_end = STORAGE_HEADER_SIZE + payload_len;
    if payload_end > region_size {
        return false;
    }

    let payload = &region[STORAGE_HEADER_SIZE..payload_end];
    if checksum32(payload) != expected_checksum {
        return false;
    }

    dst.clear();
    let copy_len = core::cmp::min(payload.len(), N);
    let _ = dst.extend_from_slice(&payload[..copy_len]);
    true
}

#[inline(never)]
#[link_section = ".data"]
unsafe fn flash_commit_region_from_ram(
    offset: usize,
    region_size: usize,
    data: &[u8],
    magic: [u8; 4],
    checksum: u32,
) {
    hal::rom_data::connect_internal_flash();
    hal::rom_data::flash_exit_xip();

    hal::rom_data::flash_range_erase(
        offset as u32,
        region_size,
        FLASH_ERASE_SIZE as u32,
        FLASH_ERASE_CMD_SECTOR,
    );

    let mut page = [0u8; FLASH_PROGRAM_SIZE];
    let mut i = 0;
    while i < FLASH_PROGRAM_SIZE {
        page[i] = 0xFF;
        i += 1;
    }

    page[0] = magic[0];
    page[1] = magic[1];
    page[2] = magic[2];
    page[3] = magic[3];

    let data_len_bytes = (data.len() as u32).to_le_bytes();
    page[4] = data_len_bytes[0];
    page[5] = data_len_bytes[1];
    page[6] = data_len_bytes[2];
    page[7] = data_len_bytes[3];

    let checksum_bytes = checksum.to_le_bytes();
    page[8] = checksum_bytes[0];
    page[9] = checksum_bytes[1];
    page[10] = checksum_bytes[2];
    page[11] = checksum_bytes[3];
    page[12] = 0;
    page[13] = 0;
    page[14] = 0;
    page[15] = 0;

    let first_chunk_len = if data.len() < (FLASH_PROGRAM_SIZE - STORAGE_HEADER_SIZE) {
        data.len()
    } else {
        FLASH_PROGRAM_SIZE - STORAGE_HEADER_SIZE
    };

    let mut first_idx = 0;
    while first_idx < first_chunk_len {
        page[STORAGE_HEADER_SIZE + first_idx] = data[first_idx];
        first_idx += 1;
    }

    hal::rom_data::flash_range_program(offset as u32, page.as_ptr(), FLASH_PROGRAM_SIZE);

    let mut written = first_chunk_len;
    let mut addr = offset + FLASH_PROGRAM_SIZE;
    while written < data.len() {
        let mut fill_idx = 0;
        while fill_idx < FLASH_PROGRAM_SIZE {
            page[fill_idx] = 0xFF;
            fill_idx += 1;
        }

        let remaining = data.len() - written;
        let chunk = if remaining < FLASH_PROGRAM_SIZE {
            remaining
        } else {
            FLASH_PROGRAM_SIZE
        };

        let mut chunk_idx = 0;
        while chunk_idx < chunk {
            page[chunk_idx] = data[written + chunk_idx];
            chunk_idx += 1;
        }

        hal::rom_data::flash_range_program(addr as u32, page.as_ptr(), FLASH_PROGRAM_SIZE);
        written += chunk;
        addr += FLASH_PROGRAM_SIZE;
    }

    hal::rom_data::flash_flush_cache();
    hal::rom_data::flash_enter_cmd_xip();
}

fn persist_error_label(err: PersistError) -> &'static str {
    match err {
        PersistError::Unsupported => "unsupported",
        PersistError::Empty => "empty",
        PersistError::TooLarge => "too_large",
        PersistError::RegionTooSmall => "region_too_small",
    }
}

fn persist_blob_to_flash(data: &[u8], target: WriteTarget) -> Result<(), PersistError> {
    let Some((offset, region_size, max_payload, magic)) = storage_layout_for(target) else {
        return Err(PersistError::Unsupported);
    };

    if data.is_empty() || data.len() > max_payload {
        if data.is_empty() {
            return Err(PersistError::Empty);
        }
        return Err(PersistError::TooLarge);
    }

    let total = STORAGE_HEADER_SIZE + data.len();
    let aligned_program_bytes = ((total + (FLASH_PROGRAM_SIZE - 1)) / FLASH_PROGRAM_SIZE) * FLASH_PROGRAM_SIZE;
    if aligned_program_bytes > region_size {
        return Err(PersistError::RegionTooSmall);
    }

    let checksum = checksum32(data);

    cortex_m::interrupt::free(|_| {
        unsafe {
            // Restore XIP through the ROM helper after programming so USB can resume reliably.
            flash_commit_region_from_ram(offset, region_size, data, magic, checksum);
        }
    });

    Ok(())
}

#[cfg(not(feature = "rescue-mode"))]
fn parse_hex_rgb(color: &str) -> Option<RGB8> {
    let s = trim_ascii_spaces(color).trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }

    let parse = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some(RGB8 {
        r: parse(0)?,
        g: parse(2)?,
        b: parse(4)?,
    })
}

#[cfg(not(feature = "rescue-mode"))]
fn parse_led_palette_from_config(config_text: &str, key: &str) -> Option<[RGB8; LED_COUNT]> {
    let key_pos = config_text.find(key)?;
    let after_key = &config_text[key_pos + key.len()..];
    let array_start_rel = after_key.find('[')?;
    let mut cursor = &after_key[array_start_rel + 1..];

    let mut parsed = [RGB8 { r: 0, g: 0, b: 0 }; LED_COUNT];
    let mut idx = 0usize;
    while idx < LED_COUNT {
        let quote_start = cursor.find('"')?;
        let color_start = quote_start + 1;
        let quote_end_rel = cursor[color_start..].find('"')?;
        let color_end = color_start + quote_end_rel;
        parsed[idx] = parse_hex_rgb(&cursor[color_start..color_end])?;
        idx += 1;
        cursor = &cursor[color_end + 1..];
    }

    Some(parsed)
}

#[cfg(not(feature = "rescue-mode"))]
fn parse_bool_from_config(config_text: &str, key: &str) -> Option<bool> {
    let key_pos = config_text.find(key)?;
    let after_key = &config_text[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    parse_bool_token(&after_key[colon_pos + 1..])
}

#[cfg(not(feature = "rescue-mode"))]
fn start_tilt_wave(active: &mut bool, step: &mut u16, led_counter: &mut u8) {
    *active = true;
    *step = 0;
    *led_counter = 0;
}

#[cfg(not(feature = "rescue-mode"))]
fn render_tilt_wave_frame(step: u16, frame: &mut [RGB8; LED_COUNT]) {
    let total_sweep_steps = core::cmp::max(1u16, TILTWAVE_MAX_STEPS / TILTWAVE_CYCLES);
    let current_cycle_step = step % total_sweep_steps;
    let wave_position = ((current_cycle_step as usize) * (LED_COUNT * 2)) / (total_sweep_steps as usize);
    let cycle_num = step / total_sweep_steps;

    let mut led_index = 0usize;
    while led_index < LED_COUNT {
        let distance = if (led_index * 2) > wave_position {
            (led_index * 2) - wave_position
        } else {
            wave_position - (led_index * 2)
        };

        let mut color_idx: usize = if distance == 0 {
            7
        } else if distance == 1 {
            5 + (current_cycle_step as usize % 3)
        } else if distance == 2 {
            3 + (current_cycle_step as usize % 2)
        } else if distance <= 4 {
            4usize.saturating_sub(distance)
        } else {
            0
        };

        if cycle_num > 0 && (led_index + step as usize) % LED_COUNT == 0 {
            color_idx = core::cmp::min(TILTWAVE_COLORS.len() - 1, color_idx + 3);
        }

        frame[led_index] = TILTWAVE_COLORS[color_idx];
        led_index += 1;
    }
}

#[cfg(not(feature = "rescue-mode"))]
fn led_index_from_name(name: &str) -> Option<usize> {
    let n = trim_ascii_spaces(name);
    if n.eq_ignore_ascii_case("strum-up") || n.eq_ignore_ascii_case("strum-up-active") {
        Some(0)
    } else if n.eq_ignore_ascii_case("strum-down") || n.eq_ignore_ascii_case("strum-down-active") {
        Some(1)
    } else if n.eq_ignore_ascii_case("orange-fret") || n.eq_ignore_ascii_case("orange-fret-pressed") {
        Some(2)
    } else if n.eq_ignore_ascii_case("blue-fret") || n.eq_ignore_ascii_case("blue-fret-pressed") {
        Some(3)
    } else if n.eq_ignore_ascii_case("yellow-fret") || n.eq_ignore_ascii_case("yellow-fret-pressed") {
        Some(4)
    } else if n.eq_ignore_ascii_case("red-fret") || n.eq_ignore_ascii_case("red-fret-pressed") {
        Some(5)
    } else if n.eq_ignore_ascii_case("green-fret") || n.eq_ignore_ascii_case("green-fret-pressed") {
        Some(6)
    } else {
        None
    }
}

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

    #[cfg(not(feature = "rescue-mode"))]
    let neopixel_pin = pins.gpio13.into_function();

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

    #[cfg(not(feature = "rescue-mode"))]
    let cd = timer.count_down();

    #[cfg(not(feature = "rescue-mode"))]
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    #[cfg(not(feature = "rescue-mode"))]
    let mut ws = Ws2812::new(
        neopixel_pin,
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        cd,
    );

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
    let mut rx_line = HeapVec::<u8, RX_LINE_CAP>::new();
    let mut tx_buf = HeapVec::<u8, TX_BUF_CAP>::new();
    let mut write_buf = HeapVec::<u8, WRITE_BUF_CAP>::new();
    let mut write_line = HeapVec::<u8, WRITE_LINE_CAP>::new();
    let mut serial_mode = SerialMode::Command;
    let mut pending_flash_commit: Option<PendingFlashCommit> = None;
    let mut pending_flash_bytes = HeapVec::<u8, WRITE_BUF_CAP>::new();
    let mut pending_write_end_marker = false;
    let mut write_line_count: u32 = 0;

    let mut config_json = HeapVec::<u8, CONFIG_BUF_CAP>::new();
    let mut presets_json = HeapVec::<u8, PRESETS_BUF_CAP>::new();
    let mut user_presets_json = HeapVec::<u8, USER_PRESETS_BUF_CAP>::new();
    copy_default(&mut config_json, DEFAULT_CONFIG_JSON);
    copy_default(&mut presets_json, DEFAULT_PRESETS_JSON);
    copy_default(&mut user_presets_json, DEFAULT_USER_PRESETS_JSON);
    let _ = load_blob_from_flash(WriteTarget::Config, &mut config_json);
    let _ = load_blob_from_flash(WriteTarget::Presets, &mut presets_json);
    let _ = load_blob_from_flash(WriteTarget::UserPresets, &mut user_presets_json);

    #[cfg(not(feature = "rescue-mode"))]
    let mut led_released_palette = LED_RELEASED;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_enabled = true;
    #[cfg(not(feature = "rescue-mode"))]
    if let Ok(config_text) = core::str::from_utf8(config_json.as_slice()) {
        if let Some(parsed_palette) = parse_led_palette_from_config(config_text, "\"released_color\"") {
            led_released_palette = parsed_palette;
        }
        if let Some(parsed_enabled) = parse_bool_from_config(config_text, "\"tilt_wave_enabled\"") {
            tilt_wave_enabled = parsed_enabled;
        }
    }

    #[cfg(not(feature = "rescue-mode"))]
    let mut led_frame = led_released_palette;
    #[cfg(not(feature = "rescue-mode"))]
    let mut preview_frame = led_released_palette;
    #[cfg(not(feature = "rescue-mode"))]
    let mut preview_active = false;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_frame = led_released_palette;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_active = false;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_step: u16 = 0;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_led_counter: u8 = 0;
    #[cfg(not(feature = "rescue-mode"))]
    let mut previous_tilt_pressed = false;
    #[cfg(not(feature = "rescue-mode"))]
    let mut tilt_wave_trigger_requested = false;

    let mut next_report_at = timer.get_counter().ticks();
    #[cfg(feature = "rescue-mode")]
    let mut phase: u8 = 0;

    loop {
        let mut classes: [&mut dyn UsbClass<UsbBus>; 2] = [&mut serial, &mut hid];
        if usb_dev.poll(&mut classes) {
            match serial.read(&mut serial_buf) {
                Ok(count) if count > 0 => {
                    for byte in &serial_buf[..count] {
                        match serial_mode {
                            SerialMode::Command => {
                                if *byte == b'\n' || *byte == b'\r' {
                                    if !rx_line.is_empty() {
                                        maybe_enter_bootsel(rx_line.as_slice());

                                        let line = core::str::from_utf8(rx_line.as_slice())
                                            .unwrap_or("");
                                        let line = trim_ascii_spaces(line);

                                        if command_requires_ack(line) {
                                            tx_push_ack(&mut tx_buf, line);
                                        }

                                        if line.eq_ignore_ascii_case("FIRMWARE_READY?")
                                            || line.eq_ignore_ascii_case("READY?")
                                        {
                                            tx_push_str(&mut tx_buf, "FIRMWARE_READY:OK\n");
                                        } else if line.eq_ignore_ascii_case("READVERSION") {
                                            tx_push_str(&mut tx_buf, "VERSION:");
                                            tx_push_str(&mut tx_buf, FW_VERSION);
                                            tx_push_str(&mut tx_buf, "\nEND\n");
                                        } else if line.eq_ignore_ascii_case("READDEVICENAME") {
                                            tx_push_str(&mut tx_buf, DEVICE_NAME);
                                            tx_push_str(&mut tx_buf, "\nEND\n");
                                        } else if line.eq_ignore_ascii_case("READUID") {
                                            tx_push_str(&mut tx_buf, DEVICE_UID);
                                            tx_push_str(&mut tx_buf, "\nEND\n");
                                        } else if let Some(path) = line.strip_prefix("READFILE:") {
                                            let trimmed = trim_ascii_spaces(path);
                                            let filename = filename_from_path(trimmed);
                                            match target_from_path(trimmed) {
                                                WriteTarget::Config => {
                                                    tx_push_file(
                                                        &mut tx_buf,
                                                        filename,
                                                        config_json.as_slice(),
                                                    );
                                                }
                                                WriteTarget::Presets => {
                                                    tx_push_file(
                                                        &mut tx_buf,
                                                        filename,
                                                        presets_json.as_slice(),
                                                    );
                                                }
                                                WriteTarget::UserPresets => {
                                                    tx_push_file(
                                                        &mut tx_buf,
                                                        filename,
                                                        user_presets_json.as_slice(),
                                                    );
                                                }
                                                WriteTarget::Unsupported => {
                                                    tx_push_str(&mut tx_buf, "START_");
                                                    tx_push_str(&mut tx_buf, filename);
                                                    tx_push_str(&mut tx_buf, "\nERROR: File not found: /");
                                                    tx_push_str(&mut tx_buf, trimmed.trim_start_matches('/'));
                                                    tx_push_str(&mut tx_buf, "\nEND_");
                                                    tx_push_str(&mut tx_buf, filename);
                                                    tx_push_str(&mut tx_buf, "\n");
                                                }
                                            }
                                        } else if let Some(path) = line.strip_prefix("WRITEFILE:") {
                                            let trimmed = trim_ascii_spaces(path);
                                            let filename = filename_from_path(trimmed);
                                            tx_push_str(&mut tx_buf, "WRITEFILE:READY:");
                                            tx_push_str(&mut tx_buf, filename);
                                            tx_push_str(&mut tx_buf, "\nSTREAM:READY:");
                                            tx_push_str(&mut tx_buf, filename);
                                            tx_push_str(&mut tx_buf, "\n");
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::WriteFile(target_from_path(trimmed));
                                        } else if line.eq_ignore_ascii_case("IMPORTUSER") {
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::ImportUser;
                                        } else if line.eq_ignore_ascii_case("READWHAMMY") {
                                            tx_push_str(&mut tx_buf, "WHAMMY:-1\n");
                                        } else if line.eq_ignore_ascii_case("READJOYSTICK") {
                                            tx_push_str(&mut tx_buf, "JOYSTICK:X:-1:Y:-1\n");
                                        } else if let Some(name) = line.strip_prefix("READPIN:") {
                                            let name = trim_ascii_spaces(name);
                                            #[cfg(not(feature = "rescue-mode"))]
                                            {
                                                let value = if name.eq_ignore_ascii_case("GREEN_FRET") {
                                                    Some(green_fret.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("RED_FRET") {
                                                    Some(red_fret.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("YELLOW_FRET") {
                                                    Some(yellow_fret.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("BLUE_FRET") {
                                                    Some(blue_fret.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("ORANGE_FRET") {
                                                    Some(orange_fret.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("STRUM_UP") {
                                                    Some(strum_up.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("STRUM_DOWN") {
                                                    Some(strum_down.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("START") {
                                                    Some(start.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("SELECT") {
                                                    Some(select.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("TILT") {
                                                    Some(tilt.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("GUIDE") {
                                                    Some(guide.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("UP") {
                                                    Some(dpad_up.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("DOWN") {
                                                    Some(dpad_down.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("LEFT") {
                                                    Some(dpad_left.is_low().unwrap_or(false))
                                                } else if name.eq_ignore_ascii_case("RIGHT") {
                                                    Some(dpad_right.is_low().unwrap_or(false))
                                                } else {
                                                    None
                                                };

                                                tx_push_str(&mut tx_buf, "PIN:");
                                                tx_push_str(&mut tx_buf, name);
                                                tx_push_str(&mut tx_buf, ":");
                                                match value {
                                                    Some(true) => tx_push_str(&mut tx_buf, "1\n"),
                                                    Some(false) => tx_push_str(&mut tx_buf, "0\n"),
                                                    None => tx_push_str(&mut tx_buf, "ERR\n"),
                                                }
                                            }
                                            #[cfg(feature = "rescue-mode")]
                                            {
                                                tx_push_str(&mut tx_buf, "PIN:");
                                                tx_push_str(&mut tx_buf, name);
                                                tx_push_str(&mut tx_buf, ":ERR\n");
                                            }
                                        } else if starts_with_ignore_ascii_case(line, "PREVIEWLED:") {
                                            #[cfg(not(feature = "rescue-mode"))]
                                            {
                                                let mut parts = line.split(':');
                                                let _ = parts.next();
                                                let led_name = parts.next().map(trim_ascii_spaces);
                                                let color = parts.next().map(trim_ascii_spaces);
                                                if let (Some(led_name), Some(color)) = (led_name, color) {
                                                    if let (Some(idx), Some(rgb)) =
                                                        (led_index_from_name(led_name), parse_hex_rgb(color))
                                                    {
                                                        preview_frame[idx] = rgb;
                                                        preview_active = true;
                                                    }
                                                }
                                            }
                                            tx_push_str(&mut tx_buf, "PREVIEWLED:OK\n");
                                        } else if starts_with_ignore_ascii_case(line, "SETLED:") {
                                            #[cfg(not(feature = "rescue-mode"))]
                                            {
                                                let mut parts = line.split(':');
                                                let _ = parts.next();
                                                if let (Some(a), Some(b), Some(c), Some(d)) =
                                                    (parts.next(), parts.next(), parts.next(), parts.next())
                                                {
                                                    let parsed = (
                                                        trim_ascii_spaces(a).parse::<usize>().ok(),
                                                        trim_ascii_spaces(b).parse::<u8>().ok(),
                                                        trim_ascii_spaces(c).parse::<u8>().ok(),
                                                        trim_ascii_spaces(d).parse::<u8>().ok(),
                                                    );
                                                    if let (Some(idx), Some(r), Some(g), Some(b)) = parsed {
                                                        if idx < LED_COUNT {
                                                            preview_frame[idx] = RGB8 { r, g, b };
                                                            preview_active = true;
                                                        }
                                                    }
                                                } else {
                                                    let mut parts2 = line.split(':');
                                                    let _ = parts2.next();
                                                    let led_name = parts2.next().map(trim_ascii_spaces);
                                                    let color = parts2.next().map(trim_ascii_spaces);
                                                    if let (Some(led_name), Some(color)) = (led_name, color) {
                                                        if let (Some(idx), Some(rgb)) =
                                                            (led_index_from_name(led_name), parse_hex_rgb(color))
                                                        {
                                                            preview_frame[idx] = rgb;
                                                            preview_active = true;
                                                        }
                                                    }
                                                }
                                            }
                                            tx_push_str(&mut tx_buf, "SETLED:OK\n");
                                        } else if line.eq_ignore_ascii_case("LEDRESTORE") {
                                            #[cfg(not(feature = "rescue-mode"))]
                                            {
                                                preview_frame = led_frame;
                                                preview_active = false;
                                            }
                                            tx_push_str(&mut tx_buf, "LEDRESTORE:OK\n");
                                        } else if line.eq_ignore_ascii_case("DEMO") {
                                            tx_push_str(&mut tx_buf, "DEMO:STARTED\n");
                                        } else if line.eq_ignore_ascii_case("TILTWAVE") {
                                            #[cfg(not(feature = "rescue-mode"))]
                                            {
                                                if tilt_wave_enabled {
                                                    tilt_wave_trigger_requested = true;
                                                }
                                            }
                                            tx_push_str(&mut tx_buf, "TILTWAVE:STARTED\n");
                                        } else if starts_with_ignore_ascii_case(line, "TILTWAVE_ENABLE:") {
                                            let value_text = line.split(':').nth(1).unwrap_or("");
                                            if let Some(enabled) = parse_bool_token(value_text) {
                                                #[cfg(not(feature = "rescue-mode"))]
                                                {
                                                    tilt_wave_enabled = enabled;
                                                    if !tilt_wave_enabled {
                                                        tilt_wave_active = false;
                                                        tilt_wave_trigger_requested = false;
                                                    }
                                                }
                                                tx_push_str(&mut tx_buf, "TILTWAVE_ENABLE:");
                                                tx_push_str(&mut tx_buf, if enabled { "true\n" } else { "false\n" });
                                            } else {
                                                tx_push_str(&mut tx_buf, "ERROR: Invalid TILTWAVE_ENABLE value\n");
                                            }
                                        } else if starts_with_ignore_ascii_case(line, "DETECTPIN:") {
                                            tx_push_str(&mut tx_buf, "PINDETECT:START:");
                                            if let Some(name) = line.split(':').nth(1) {
                                                tx_push_str(&mut tx_buf, trim_ascii_spaces(name));
                                            }
                                            tx_push_str(&mut tx_buf, "\n");
                                        } else if starts_with_ignore_ascii_case(line, "SAVEPIN:") {
                                            tx_push_str(&mut tx_buf, "PINDETECT:SAVED\n");
                                        } else if line.eq_ignore_ascii_case("CANCELPINDETECT") {
                                            tx_push_str(&mut tx_buf, "PINDETECT:CANCELLED\n");
                                        } else if starts_with_ignore_ascii_case(line, "MKDIR:") {
                                            tx_push_str(&mut tx_buf, "MKDIR:SUCCESS\n");
                                        } else if line.eq_ignore_ascii_case("REBOOT") {
                                            tx_push_str(&mut tx_buf, "Rebooting...\n");
                                            SCB::sys_reset();
                                        } else if line.eq_ignore_ascii_case("REBOOTBOOTSEL") {
                                            tx_push_str(&mut tx_buf, "Rebooting to BOOTSEL mode...\n");
                                            hal::rom_data::reset_to_usb_boot(0, 0);
                                        }
                                    }
                                    rx_line.clear();
                                    continue;
                                }

                                if rx_line.len() < RX_LINE_CAP {
                                    let _ = rx_line.push(*byte);
                                }
                            }
                            SerialMode::WriteFile(target) => {
                                if *byte == b'\r' {
                                    continue;
                                }

                                if *byte == b'\n' {
                                    let line = core::str::from_utf8(write_line.as_slice()).unwrap_or("");
                                    let line = trim_ascii_spaces(line);

                                    if line.eq_ignore_ascii_case("END") {
                                        match target {
                                            WriteTarget::Config | WriteTarget::Presets | WriteTarget::UserPresets => {
                                                pending_flash_bytes.clear();
                                                let _ = pending_flash_bytes.extend_from_slice(write_buf.as_slice());
                                                pending_flash_commit = Some(PendingFlashCommit::WriteFile(target));
                                                pending_write_end_marker = true;
                                            }
                                            WriteTarget::Unsupported => {
                                                tx_push_str(
                                                    &mut tx_buf,
                                                    "ERROR: Failed to write unsupported file target\n",
                                                );
                                            }
                                        }

                                        write_line.clear();
                                        write_line_count = 0;
                                        serial_mode = SerialMode::Command;
                                    } else {
                                        if write_buf.extend_from_slice(write_line.as_slice()).is_err() {
                                            tx_push_str(&mut tx_buf, "ERROR: WRITEFILE buffer append failed\n");
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::Command;
                                            continue;
                                        }
                                        if write_buf.push(b'\n').is_err() {
                                            tx_push_str(&mut tx_buf, "ERROR: WRITEFILE buffer newline append failed\n");
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::Command;
                                            continue;
                                        }
                                        write_line_count = write_line_count.wrapping_add(1);
                                        if write_line_count % 16 == 0 {
                                            tx_push_str(&mut tx_buf, "WRITEFILE:LINES:");
                                            tx_push_u32(&mut tx_buf, write_line_count);
                                            tx_push_str(&mut tx_buf, "\n");
                                        }
                                        write_line.clear();
                                    }
                                } else if write_line.len() < WRITE_LINE_CAP {
                                    if write_line.push(*byte).is_err() {
                                        tx_push_str(&mut tx_buf, "ERROR: WRITEFILE line append failed\n");
                                        write_buf.clear();
                                        write_line.clear();
                                        write_line_count = 0;
                                        serial_mode = SerialMode::Command;
                                    }
                                } else {
                                    tx_push_str(&mut tx_buf, "ERROR: WRITEFILE line too long\n");
                                    write_buf.clear();
                                    write_line.clear();
                                    write_line_count = 0;
                                    serial_mode = SerialMode::Command;
                                }
                            }
                            SerialMode::ImportUser => {
                                if *byte == b'\r' {
                                    continue;
                                }

                                if *byte == b'\n' {
                                    let line = core::str::from_utf8(write_line.as_slice()).unwrap_or("");
                                    let line = trim_ascii_spaces(line);
                                    if line.eq_ignore_ascii_case("END") {
                                        pending_flash_bytes.clear();
                                        let _ = pending_flash_bytes.extend_from_slice(write_buf.as_slice());
                                        pending_flash_commit = Some(PendingFlashCommit::ImportUser);
                                        write_buf.clear();
                                        write_line.clear();
                                        write_line_count = 0;
                                        serial_mode = SerialMode::Command;
                                    } else {
                                        if write_buf.extend_from_slice(write_line.as_slice()).is_err() {
                                            tx_push_str(&mut tx_buf, "ERROR: IMPORTUSER buffer append failed\n");
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::Command;
                                            continue;
                                        }
                                        if write_buf.push(b'\n').is_err() {
                                            tx_push_str(&mut tx_buf, "ERROR: IMPORTUSER buffer newline append failed\n");
                                            write_buf.clear();
                                            write_line.clear();
                                            write_line_count = 0;
                                            serial_mode = SerialMode::Command;
                                            continue;
                                        }
                                        write_line_count = write_line_count.wrapping_add(1);
                                        write_line.clear();
                                    }
                                } else if write_line.len() < WRITE_LINE_CAP {
                                    if write_line.push(*byte).is_err() {
                                        tx_push_str(&mut tx_buf, "ERROR: IMPORTUSER line append failed\n");
                                        write_buf.clear();
                                        write_line.clear();
                                        write_line_count = 0;
                                        serial_mode = SerialMode::Command;
                                    }
                                } else {
                                    tx_push_str(&mut tx_buf, "ERROR: IMPORTUSER line too long\n");
                                    write_buf.clear();
                                    write_line.clear();
                                    write_line_count = 0;
                                    serial_mode = SerialMode::Command;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            flush_tx(&mut serial, &mut tx_buf);
        }

        if pending_write_end_marker {
            tx_push_str(&mut tx_buf, "WRITEFILE:END\n");
            pending_write_end_marker = false;
        }

        if tx_buf.is_empty() {
            if let Some(commit) = pending_flash_commit.take() {
                match commit {
                PendingFlashCommit::WriteFile(target) => {
                    let persist_result = persist_blob_to_flash(pending_flash_bytes.as_slice(), target);
                    match (target, persist_result) {
                        (WriteTarget::Config, Ok(())) => {
                            config_json.clear();
                            let _ = config_json.extend_from_slice(pending_flash_bytes.as_slice());
                            #[cfg(not(feature = "rescue-mode"))]
                            if let Ok(config_text) = core::str::from_utf8(config_json.as_slice()) {
                                if let Some(parsed_enabled) =
                                    parse_bool_from_config(config_text, "\"tilt_wave_enabled\"")
                                {
                                    tilt_wave_enabled = parsed_enabled;
                                    if !tilt_wave_enabled {
                                        tilt_wave_active = false;
                                        tilt_wave_trigger_requested = false;
                                    }
                                }
                            }
                            tx_push_str(&mut tx_buf, "File /config.json written (flash)\n");
                        }
                        (WriteTarget::Presets, Ok(())) => {
                            presets_json.clear();
                            let _ = presets_json.extend_from_slice(pending_flash_bytes.as_slice());
                            tx_push_str(&mut tx_buf, "File /presets.json written (flash)\n");
                        }
                        (WriteTarget::UserPresets, Ok(())) => {
                            user_presets_json.clear();
                            let _ = user_presets_json.extend_from_slice(pending_flash_bytes.as_slice());
                            tx_push_str(&mut tx_buf, "File /user_presets.json written (flash)\n");
                        }
                        (WriteTarget::Config, Err(err)) => {
                            tx_push_str(&mut tx_buf, "ERROR: Failed to persist /config.json:");
                            tx_push_str(&mut tx_buf, persist_error_label(err));
                            tx_push_str(&mut tx_buf, "\n");
                        }
                        (WriteTarget::Presets, Err(err)) => {
                            tx_push_str(&mut tx_buf, "ERROR: Failed to persist /presets.json:");
                            tx_push_str(&mut tx_buf, persist_error_label(err));
                            tx_push_str(&mut tx_buf, "\n");
                        }
                        (WriteTarget::UserPresets, Err(err)) => {
                            tx_push_str(&mut tx_buf, "ERROR: Failed to persist /user_presets.json:");
                            tx_push_str(&mut tx_buf, persist_error_label(err));
                            tx_push_str(&mut tx_buf, "\n");
                        }
                        (WriteTarget::Unsupported, _) => {
                            tx_push_str(&mut tx_buf, "ERROR: Failed to write unsupported file target\n");
                        }
                    }
                    pending_flash_bytes.clear();
                }
                PendingFlashCommit::ImportUser => {
                    match persist_blob_to_flash(pending_flash_bytes.as_slice(), WriteTarget::UserPresets) {
                        Ok(()) => {
                        user_presets_json.clear();
                        let _ = user_presets_json.extend_from_slice(pending_flash_bytes.as_slice());
                        tx_push_str(&mut tx_buf, "Merged into /user_presets.json (flash)\n");
                        }
                        Err(err) => {
                            tx_push_str(&mut tx_buf, "ERROR: Failed to persist merged user_presets.json:");
                            tx_push_str(&mut tx_buf, persist_error_label(err));
                            tx_push_str(&mut tx_buf, "\n");
                        }
                    }
                    pending_flash_bytes.clear();
                }
            }

                flush_tx(&mut serial, &mut tx_buf);
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

            if tilt_wave_enabled && (tilt_wave_trigger_requested || (tilt_pressed && !previous_tilt_pressed)) {
                start_tilt_wave(
                    &mut tilt_wave_active,
                    &mut tilt_wave_step,
                    &mut tilt_wave_led_counter,
                );
            }
            tilt_wave_trigger_requested = false;
            previous_tilt_pressed = tilt_pressed;

            if tilt_wave_active {
                tilt_wave_led_counter = tilt_wave_led_counter.wrapping_add(1);
                if tilt_wave_led_counter >= 2 {
                    tilt_wave_led_counter = 0;
                    if tilt_wave_step >= TILTWAVE_MAX_STEPS {
                        tilt_wave_active = false;
                    } else {
                        render_tilt_wave_frame(tilt_wave_step, &mut tilt_wave_frame);
                        tilt_wave_step = tilt_wave_step.wrapping_add(1);
                    }
                }
            }

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

            led_frame[0] = if strum_up_pressed { LED_PRESSED[0] } else { led_released_palette[0] };
            led_frame[1] = if strum_down_pressed { LED_PRESSED[1] } else { led_released_palette[1] };
            led_frame[2] = if orange_pressed { LED_PRESSED[2] } else { led_released_palette[2] };
            led_frame[3] = if blue_pressed { LED_PRESSED[3] } else { led_released_palette[3] };
            led_frame[4] = if yellow_pressed { LED_PRESSED[4] } else { led_released_palette[4] };
            led_frame[5] = if red_pressed { LED_PRESSED[5] } else { led_released_palette[5] };
            led_frame[6] = if green_pressed { LED_PRESSED[6] } else { led_released_palette[6] };

            if preview_active {
                let _ = ws.write(preview_frame.iter().copied());
            } else if tilt_wave_active {
                let _ = ws.write(tilt_wave_frame.iter().copied());
            } else {
                let _ = ws.write(led_frame.iter().copied());
                preview_frame = led_frame;
            }

            let _ = hid.push_input(&report);
            next_report_at = now + REPORT_INTERVAL_US;
            }
        }
    }
}
