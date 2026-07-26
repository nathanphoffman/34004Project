// PS/2 keyboard receiver — two different implementations behind the
// same kb_read_key()/ps2_configure functions, picked by the same
// rp2350/rp2040 Cargo feature this project already uses elsewhere in
// kernel/ for real-chip-vs-Wokwi-stand-in differences (see state.rs's
// IMAGE_DEF/BOOT2_FIRMWARE for another example of the same pattern).
//
// rp2350 (real hardware): a GPIO falling-edge interrupt on CLK. This is
// the correct, power-efficient way to do this, and interrupts work
// completely fine on real silicon — hw_init() also puts the CPU to
// sleep between them via kernel_idle() (see hardware.rs).
//
// rp2040 (Wokwi stand-in): CLK is polled instead, from kb_read_key(),
// which runs once per application/app.deor loop iteration. This is not
// a simplification for its own sake — it's a direct response to a real,
// investigated problem: every version of this file that used an
// interrupt (even a single one-shot GPIO interrupt doing minimal work)
// caused Wokwi's simulation speed to degrade and stay degraded for the
// rest of the run, not just during the brief moment the interrupt fired.
// That points at interrupt *dispatch* itself being expensive in Wokwi's
// rp2040js simulator (consistent with a documented Wokwi issue on their
// interrupt timing differing from real hardware — wokwi/wokwi-features
// #924), not at anything this code does inside a handler. Since real
// hardware doesn't have that problem, real hardware keeps the interrupt;
// Wokwi gets a polling fallback instead of fighting the simulator.
//
// Both paths share: the 11-bit PS/2 frame format (start bit, 8 data
// bits LSB-first, parity, stop bit), via shift_in_bit() below, and Scan
// Code Set 2 decoding. Only *how a bit gets noticed* differs — a
// hardware interrupt vs. a software comparison against the last-seen
// CLK level. See keyboard_sim.rs for the same rp2350/rp2040 split
// applied to the Wokwi-only simulated-keystroke injector.

fn decode_set2(scancode: u8) -> Option<char> {
    match scancode {
        0x1C => Some('a'), 0x32 => Some('b'), 0x21 => Some('c'), 0x23 => Some('d'),
        0x24 => Some('e'), 0x2B => Some('f'), 0x34 => Some('g'), 0x33 => Some('h'),
        0x43 => Some('i'), 0x3B => Some('j'), 0x42 => Some('k'), 0x4B => Some('l'),
        0x3A => Some('m'), 0x31 => Some('n'), 0x44 => Some('o'), 0x4D => Some('p'),
        0x15 => Some('q'), 0x2D => Some('r'), 0x1B => Some('s'), 0x2C => Some('t'),
        0x3C => Some('u'), 0x2A => Some('v'), 0x1D => Some('w'), 0x22 => Some('x'),
        0x35 => Some('y'), 0x1A => Some('z'),
        0x45 => Some('0'), 0x16 => Some('1'), 0x1E => Some('2'), 0x26 => Some('3'),
        0x25 => Some('4'), 0x2E => Some('5'), 0x36 => Some('6'), 0x3D => Some('7'),
        0x3E => Some('8'), 0x46 => Some('9'),
        0x29 => Some(' '), 0x5A => Some('\n'), 0x66 => Some('\u{8}'), 0x0D => Some('\t'),
        0x76 => Some('\u{1b}'),
        _ => None,
    }
}

/// Shared by both paths below: fold one newly-observed bit into the
/// in-progress frame: shift it in, and once 11 bits have arrived,
/// validate start/stop, track the break (0xF0) / extended (0xE0)
/// prefixes (which arrive as their own separate frames and must persist
/// until the frame after them), and queue the decoded character. Not
/// mapping break/extended codes to a character is deliberate — this is
/// meant for typing text, not modifier/arrow keys.
fn shift_in_bit(state: &mut Ps2State, bit: u16, cs: critical_section::CriticalSection<'_>) {
    state.bits |= bit << state.count;
    state.count += 1;

    if state.count < 11 {
        return;
    }

    let frame = state.bits;
    let start_ok = frame & 0x1 == 0;
    let stop_ok = (frame >> 10) & 0x1 == 1;
    let byte = ((frame >> 1) & 0xFF) as u8;
    state.bits = 0;
    state.count = 0;

    if !start_ok || !stop_ok {
        return;
    }

    if byte == 0xF0 {
        state.pending_break = true;
    } else if byte == 0xE0 {
        state.pending_extended = true;
    } else {
        let was_break = state.pending_break;
        let was_extended = state.pending_extended;
        state.pending_break = false;
        state.pending_extended = false;

        if !was_break && !was_extended {
            if let Some(c) = decode_set2(byte) {
                KEY_QUEUE.borrow(cs).borrow_mut().push_back(c);
            }
        }
    }
}

// ---------------------------------------------------------------------
// rp2350 (real hardware): interrupt-driven.
// ---------------------------------------------------------------------

/// Arm the CLK falling-edge interrupt. Called from hw_init() with pins
/// it already owns — `pac::Peripherals::take()` only succeeds once, so
/// this can't grab its own.
#[cfg(feature = "rp2350")]
fn ps2_configure(mut clk: InPin, data: InPin) {
    clk.clear_interrupt(hal::gpio::Interrupt::EdgeLow);
    clk.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);

    critical_section::with(|cs| {
        PS2_PINS.borrow(cs).replace(Some(Ps2Pins { clk, data }));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::IO_IRQ_BANK0);
    }
}

#[cfg(feature = "rp2350")]
#[interrupt]
fn IO_IRQ_BANK0() {
    critical_section::with(|cs| {
        let mut pins_slot = PS2_PINS.borrow(cs).borrow_mut();
        let Some(pins) = pins_slot.as_mut() else { return };

        if !pins.clk.interrupt_status(hal::gpio::Interrupt::EdgeLow) {
            return;
        }
        pins.clk.clear_interrupt(hal::gpio::Interrupt::EdgeLow);

        let bit: u16 = if pins.data.is_high().unwrap_or(false) { 1 } else { 0 };
        let mut state = PS2_STATE.borrow(cs).borrow_mut();
        shift_in_bit(&mut state, bit, cs);
    });
}

/// No-op: on real hardware, bits arrive via the interrupt above, not by
/// being polled for.
#[cfg(feature = "rp2350")]
fn ps2_poll_once(_cs: critical_section::CriticalSection<'_>) {}

// ---------------------------------------------------------------------
// rp2040 (Wokwi stand-in): polled. See the file-level comment for why.
// ---------------------------------------------------------------------

/// Stash the CLK/DATA pins for ps2_poll_once to read every call.
#[cfg(feature = "rp2040")]
fn ps2_configure(clk: InPin, data: InPin) {
    critical_section::with(|cs| {
        PS2_PINS.borrow(cs).replace(Some(Ps2Pins { clk, data }));
    });
}

/// Check CLK once; a falling edge is "was high last poll, low now" —
/// software edge detection, since there's no hardware interrupt to tell
/// us. PS/2's clock rate (~10-16kHz) is far slower than kb_read_key()
/// gets called (once per app_main() loop iteration, with no delay in
/// between), so no edges are missed despite not being interrupt-driven.
#[cfg(feature = "rp2040")]
fn ps2_poll_once(cs: critical_section::CriticalSection<'_>) {
    let mut pins_slot = PS2_PINS.borrow(cs).borrow_mut();
    let Some(pins) = pins_slot.as_mut() else { return };

    let clk_high = pins.clk.is_high().unwrap_or(true);
    let mut state = PS2_STATE.borrow(cs).borrow_mut();
    let falling_edge = state.last_clk_high && !clk_high;
    state.last_clk_high = clk_high;

    if !falling_edge {
        return;
    }

    let bit: u16 = if pins.data.is_high().unwrap_or(false) { 1 } else { 0 };
    shift_in_bit(&mut state, bit, cs);
}

// ---------------------------------------------------------------------

/// Poll (Wokwi) or no-op (real hardware — the interrupt above already
/// did the work) the PS/2 line and the sim injector, then pop the
/// oldest queued keystroke — or `""` if none is waiting.
pub fn kb_read_key() -> String {
    critical_section::with(|cs| {
        ps2_poll_once(cs);
        keyboard_sim_poll_once(cs);
        KEY_QUEUE.borrow(cs).borrow_mut().pop_front()
    })
    .map(|c| c.to_string())
    .unwrap_or_default()
}
