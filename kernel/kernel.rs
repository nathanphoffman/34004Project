// RP2350 (Pico 2) hardware bring-up and a minimal bit-banged HD44780
// 4-bit driver for three shared-bus 20x4 LCD panels, split across this
// directory: state.rs (types + statics), lcd.rs (bit-bang driver),
// hardware.rs (hw_init/hw_print_str — the two entry points Deor calls).
//
// Pulled in verbatim into build/main.rs via kernel/boot.deor's
// include!("../kernel/kernel.rs") — see main.deor and
// docs/interop.md#external-rs-files. The nested include!s below resolve
// relative to this file's own directory, not to build/main.rs.

use core::cell::RefCell;
use core::mem::MaybeUninit;
use critical_section::Mutex;
use embedded_hal::digital::OutputPin;
use hal::Clock;

include!("state.rs");
include!("lcd.rs");
include!("hardware.rs");
