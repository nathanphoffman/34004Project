// RP2350 (Pico 2) hardware bring-up and a minimal bit-banged HD44780
// 4-bit driver for three shared-bus 20x4 LCD panels.
//
// Included verbatim (mid-file, hence plain `//` and not `//!` module docs)
// into the Deor-generated `build/main.rs` via `include!("../kernel.rs")`
// — see main.deor. Deor calls into this file only through `hw_init()` and
// `hw_print_str()`, both plain-data (int/string) calls, so nothing
// hardware-shaped ever has to cross the Deor/Rust boundary.

use core::cell::RefCell;
use core::mem::MaybeUninit;
use critical_section::Mutex;
use embedded_hal::digital::OutputPin;
use hal::Clock;

/// Tell the RP2350 Boot ROM about this application.
#[cfg(feature = "rp2350")]
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// RP2040 needs an explicit second-stage bootloader blob instead — required
/// only for the Wokwi (Pico 1) simulation build.
#[cfg(feature = "rp2040")]
#[link_section = ".boot2"]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

/// External crystal on the Raspberry Pi Pico 2 board.
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Deor strings transpile to heap-allocated `String`, so `alloc` needs a
/// global allocator even for this tiny amount of text.
const HEAP_SIZE: usize = 4096;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[global_allocator]
static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

type OutPin = hal::gpio::Pin<hal::gpio::DynPinId, hal::gpio::FunctionSioOutput, hal::gpio::PullDown>;

struct Lcd {
    rs: OutPin,
    en: OutPin,
}

struct DataBus {
    d4: OutPin,
    d5: OutPin,
    d6: OutPin,
    d7: OutPin,
}

struct Hardware {
    bus: DataBus,
    lcd1: Lcd,
    lcd2: Lcd,
    lcd3: Lcd,
    delay: cortex_m::delay::Delay,
}

static HARDWARE: Mutex<RefCell<Option<Hardware>>> = Mutex::new(RefCell::new(None));

fn pulse_enable(en: &mut OutPin, delay: &mut cortex_m::delay::Delay) {
    en.set_high().unwrap();
    delay.delay_us(1);
    en.set_low().unwrap();
    delay.delay_us(50);
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
    delay.delay_us(50);
}

fn lcd_command(bus: &mut DataBus, lcd: &mut Lcd, cmd: u8, delay: &mut cortex_m::delay::Delay) {
    write_byte(bus, lcd, cmd, false, delay);
}

fn lcd_init(bus: &mut DataBus, lcd: &mut Lcd, delay: &mut cortex_m::delay::Delay) {
    // HD44780 4-bit "reset by instruction" sequence — brings the
    // controller to a known state regardless of its power-on condition.
    delay.delay_ms(50);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_ms(5);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_us(150);
    write_nibble(bus, &mut lcd.en, 0x03, delay);
    delay.delay_us(150);
    write_nibble(bus, &mut lcd.en, 0x02, delay); // switch to 4-bit mode

    lcd_command(bus, lcd, 0x28, delay); // function set: 4-bit, 2-line, 5x8 font
    lcd_command(bus, lcd, 0x0C, delay); // display on, cursor off, blink off
    lcd_command(bus, lcd, 0x01, delay); // clear display
    delay.delay_ms(2);
    lcd_command(bus, lcd, 0x06, delay); // entry mode: increment, no shift
}

fn lcd_print_line(bus: &mut DataBus, lcd: &mut Lcd, text: &str, delay: &mut cortex_m::delay::Delay) {
    lcd_command(bus, lcd, 0x80, delay); // row 0, column 0
    for byte in text.bytes().take(20) {
        write_byte(bus, lcd, byte, true, delay);
    }
}

/// Bring up clocks, GPIO, and all three LCDs. Must run before `hw_print_str`.
pub fn hw_init() {
    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    let mut bus = DataBus {
        d4: pins.gpio2.into_push_pull_output().into_dyn_pin(),
        d5: pins.gpio3.into_push_pull_output().into_dyn_pin(),
        d6: pins.gpio4.into_push_pull_output().into_dyn_pin(),
        d7: pins.gpio5.into_push_pull_output().into_dyn_pin(),
    };

    let mut lcd1 = Lcd {
        rs: pins.gpio0.into_push_pull_output().into_dyn_pin(),
        en: pins.gpio1.into_push_pull_output().into_dyn_pin(),
    };
    let mut lcd2 = Lcd {
        rs: pins.gpio6.into_push_pull_output().into_dyn_pin(),
        en: pins.gpio7.into_push_pull_output().into_dyn_pin(),
    };
    let mut lcd3 = Lcd {
        rs: pins.gpio8.into_push_pull_output().into_dyn_pin(),
        en: pins.gpio9.into_push_pull_output().into_dyn_pin(),
    };

    // Backlight anodes, each through a series resistor (R1/R2/R3 in the
    // Wokwi diagram) — drive high to turn the backlights on.
    let mut backlight1 = pins.gpio14.into_push_pull_output();
    let mut backlight2 = pins.gpio15.into_push_pull_output();
    let mut backlight3 = pins.gpio16.into_push_pull_output();
    backlight1.set_high().unwrap();
    backlight2.set_high().unwrap();
    backlight3.set_high().unwrap();

    lcd_init(&mut bus, &mut lcd1, &mut delay);
    lcd_init(&mut bus, &mut lcd2, &mut delay);
    lcd_init(&mut bus, &mut lcd3, &mut delay);

    critical_section::with(|cs| {
        HARDWARE.borrow(cs).replace(Some(Hardware { bus, lcd1, lcd2, lcd3, delay }));
    });
}

/// Print `text` to row 0 of the given display (1, 2, or 3).
pub fn hw_print_str(display: i64, text: &str) {
    critical_section::with(|cs| {
        let mut slot = HARDWARE.borrow(cs).borrow_mut();
        let hw = slot.as_mut().unwrap();
        // Match ergonomics splits this into disjoint `&mut` borrows of
        // each field, so `bus` and `delay` stay usable alongside whichever
        // `lcd*` we pick below.
        let Hardware { bus, lcd1, lcd2, lcd3, delay } = hw;
        let lcd = match display {
            1 => lcd1,
            2 => lcd2,
            _ => lcd3,
        };
        lcd_print_line(bus, lcd, text, delay);
    });
}
