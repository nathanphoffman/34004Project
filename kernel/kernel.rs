// RP2350 (Pico 2) hardware bring-up, a minimal bit-banged HD44780 4-bit
// driver for three shared-bus 20x4 LCD panels, and a PS/2 keyboard
// receiver — split across this directory: state.rs (types + statics),
// lcd.rs (bit-bang driver), hardware.rs (hw_init/hw_print_str — the
// entry points Deor calls), keyboard.rs (real PS/2 receiver, polled),
// keyboard_sim.rs (Wokwi-only simulated keystrokes, also polled).
//
// Deliberately no interrupts anywhere in this crate — see keyboard.rs
// for why. Everything runs from application/app.deor's plain poll loop.
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

include!("state.rs");
include!("lcd.rs");
include!("hardware.rs");
include!("keyboard.rs");
include!("keyboard_sim.rs");
