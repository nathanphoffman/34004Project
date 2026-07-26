// RP2350 (Pico 2) hardware bring-up, a minimal bit-banged HD44780 4-bit
// driver for three shared-bus 20x4 LCD panels, and a PS/2 keyboard
// receiver — split across this directory: state.rs (types + statics),
// lcd.rs (bit-bang driver), hardware.rs (hw_init/hw_print_str/
// kernel_idle — the entry points Deor calls), keyboard.rs (real PS/2
// receiver), keyboard_sim.rs (Wokwi-only simulated keystrokes).
//
// keyboard.rs and keyboard_sim.rs each compile one of two
// implementations depending on the rp2350/rp2040 feature — real
// hardware uses GPIO/timer interrupts (and kernel_idle() sleeps between
// them via wfi()), Wokwi uses plain polling from application/app.deor's
// loop instead, because Wokwi's rp2040js simulator gets dramatically
// slower once any interrupt fires — see the comment at the top of
// keyboard.rs for the full investigation.
//
// Pulled in verbatim into build/main.rs via kernel/boot.deor's
// include!("../kernel/kernel.rs") — see main.deor and
// docs/interop.md#external-rs-files. The nested include!s below resolve
// relative to this file's own directory, not to build/main.rs.

use core::cell::RefCell;
use core::mem::MaybeUninit;
use alloc::collections::VecDeque;
use critical_section::Mutex;
use embedded_hal::digital::{InputPin, OutputPin};
use hal::Clock;
#[cfg(feature = "rp2350")]
use hal::pac::interrupt;

include!("state.rs");
include!("lcd.rs");
include!("hardware.rs");
include!("keyboard.rs");
include!("keyboard_sim.rs");
