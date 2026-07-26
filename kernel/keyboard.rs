// PS/2 keyboard receiver. The keyboard drives CLK; on each falling edge
// the host samples DATA and shifts it into an 11-bit frame (start bit,
// 8 data bits LSB-first, parity, stop bit). Decoded characters land in
// KEY_QUEUE for xk_read_key() to drain — see lib/keyboard.deor.
//
// Scan Code Set 2 only covers plain alphanumeric/space/enter/backspace/
// tab/escape here; break codes (0xF0 prefix, key-up) and the 0xE0
// extended prefix are recognized and consumed but not mapped to a
// character, since this is meant for typing text, not modifier/arrow
// keys. Table verified against the OSDev PS/2 Keyboard reference.

/// Wire up the CLK/DATA pins and enable the interrupt that drives the
/// bit-shift state machine. Called from hw_init() with pins it already
/// owns — `pac::Peripherals::take()` only succeeds once, so this can't
/// grab its own.
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
    });
}

/// Pop the oldest queued keystroke, or `""` if none is waiting.
pub fn kb_read_key() -> String {
    critical_section::with(|cs| KEY_QUEUE.borrow(cs).borrow_mut().pop_front())
        .map(|c| c.to_string())
        .unwrap_or_default()
}
