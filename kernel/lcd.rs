// Bit-banged HD44780 4-bit driver, shared by all three panels via the
// types in state.rs (DataBus/Lcd).
//
// The settle/reset delays below are HD44780 datasheet timing — real
// hardware needs them. Shrunk way down for the Wokwi build only: this
// combination (shrunk delays here + the pause at the end of hw_init(),
// see hardware.rs) is the one configuration actually confirmed working
// end-to-end in Wokwi, after a lot of back-and-forth. Several
// "principled" reversions away from it (correct-but-slower delays here,
// no pause in hardware.rs) each made things worse (garbled characters,
// then no text at all) rather than better, so: known-working
// configuration wins over theory until there's a real explanation.
#[cfg(feature = "rp2040")]
const NIBBLE_SETTLE_US: u32 = 1;
#[cfg(not(feature = "rp2040"))]
const NIBBLE_SETTLE_US: u32 = 50;

#[cfg(feature = "rp2040")]
const BYTE_SETTLE_US: u32 = 1;
#[cfg(not(feature = "rp2040"))]
const BYTE_SETTLE_US: u32 = 50;

#[cfg(feature = "rp2040")]
const RESET_MS: u32 = 1;
#[cfg(not(feature = "rp2040"))]
const RESET_MS: u32 = 50;

#[cfg(feature = "rp2040")]
const RESET_SETTLE_MS: u32 = 1;
#[cfg(not(feature = "rp2040"))]
const RESET_SETTLE_MS: u32 = 5;

#[cfg(feature = "rp2040")]
const RESET_SETTLE_US: u32 = 5;
#[cfg(not(feature = "rp2040"))]
const RESET_SETTLE_US: u32 = 150;

#[cfg(feature = "rp2040")]
const CLEAR_MS: u32 = 1;
#[cfg(not(feature = "rp2040"))]
const CLEAR_MS: u32 = 2;

fn pulse_enable(en: &mut OutPin, delay: &mut cortex_m::delay::Delay) {
    en.set_high().unwrap();
    delay.delay_us(1);
    en.set_low().unwrap();
    delay.delay_us(NIBBLE_SETTLE_US);
}

fn write_nibble(bus: &mut DataBus, en: &mut OutPin, nibble: u8, delay: &mut cortex_m::delay::Delay) {
    if nibble & 0x1 != 0 { bus.d4.set_high().unwrap(); } else { bus.d4.set_low().unwrap(); }
    if nibble & 0x2 != 0 { bus.d5.set_high().unwrap(); } else { bus.d5.set_low().unwrap(); }
    if nibble & 0x4 != 0 { bus.d6.set_high().unwrap(); } else { bus.d6.set_low().unwrap(); }
    if nibble & 0x8 != 0 { bus.d7.set_high().unwrap(); } else { bus.d7.set_low().unwrap(); }
    pulse_enable(en, delay);
}

fn write_byte(bus: &mut DataBus, lcd: &mut Lcd, value: u8, is_data: bool, delay: &mut cortex_m::delay::Delay) {
    if is_data { lcd.rs.set_high().unwrap(); } else { lcd.rs.set_low().unwrap(); }
    write_nibble(bus, &mut lcd.en, value >> 4, delay);
    write_nibble(bus, &mut lcd.en, value & 0x0F, delay);
    delay.delay_us(BYTE_SETTLE_US);
}

fn lcd_command(bus: &mut DataBus, lcd: &mut Lcd, cmd: u8, delay: &mut cortex_m::delay::Delay) {
    write_byte(bus, lcd, cmd, false, delay);
}

fn lcd_init(bus: &mut DataBus, lcd: &mut Lcd, delay: &mut cortex_m::delay::Delay) {
    // HD44780 4-bit "reset by instruction" sequence — brings the
    // controller to a known state regardless of its power-on condition.
    delay.delay_ms(RESET_MS);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_ms(RESET_SETTLE_MS);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_us(RESET_SETTLE_US);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_us(RESET_SETTLE_US);
    write_nibble(bus, &mut lcd.en, 0x02, delay); // switch to 4-bit mode

    lcd_command(bus, lcd, 0x28, delay); // function set: 4-bit, 2-line, 5x8 font
    lcd_command(bus, lcd, 0x0C, delay); // display on, cursor off, blink off
    lcd_command(bus, lcd, 0x01, delay); // clear display
    delay.delay_ms(CLEAR_MS);
    lcd_command(bus, lcd, 0x06, delay); // entry mode: increment, no shift
}

/// DDRAM start address for each row of a 20x4 HD44780 display. The
/// controller only understands "1 line" or "2 lines" at the function-set
/// level (lcd_init sends 0x28, 2-line, same as always) — 4 physical rows
/// are really 2 controller lines of 40 characters each, split in half,
/// so row addressing needs this offset table instead of a simple stride.
fn row_address(row: u8) -> u8 {
    match row {
        0 => 0x00,
        1 => 0x40,
        2 => 0x14,
        _ => 0x54,
    }
}

/// Writes exactly 20 characters every time, padding with spaces past the
/// end of `text` — a terminal input line shrinks as you backspace, and
/// without padding, characters left over from a longer previous line
/// would stay stuck on screen instead of getting overwritten.
fn lcd_print_line(bus: &mut DataBus, lcd: &mut Lcd, row: u8, text: &str, delay: &mut cortex_m::delay::Delay) {
    lcd_command(bus, lcd, 0x80 | row_address(row), delay);
    let mut written = 0;
    for byte in text.bytes().take(20) {
        write_byte(bus, lcd, byte, true, delay);
        written += 1;
    }
    for _ in written..20 {
        write_byte(bus, lcd, b' ', true, delay);
    }
}
