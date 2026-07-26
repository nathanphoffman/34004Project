// Wokwi/rp2040 stand-in only. Wokwi can't simulate a real PS/2 keyboard,
// so this polls a free-running timer and, once a few seconds have
// elapsed since boot, dumps a canned string into the same KEY_QUEUE the
// real PS/2 receiver (keyboard.rs) fills — app-level code draining
// xk_read_key() can't tell the two apart, and still pops it one
// character per call. Compiled to a no-op on real hardware (feature =
// "rp2350"): kb_read_key() there only ever sees genuine keystrokes.
//
// Deliberately polled, not interrupt-driven — an earlier version used a
// hardware timer alarm interrupt. Every fix that kept an interrupt
// architecture (one-shot alarm, display batching, shrunk LCD delays)
// failed to resolve a persistent Wokwi slowdown, so interrupts came out
// entirely — see keyboard.rs for the same change on the real receiver.

#[cfg(feature = "rp2040")]
const SIM_KEYSTROKES: &str = "hello world\n";
#[cfg(feature = "rp2040")]
const SIM_BOOT_DELAY_US: u32 = 5_000_000;

#[cfg(feature = "rp2040")]
static SIM_TIMER: Mutex<RefCell<Option<hal::Timer>>> = Mutex::new(RefCell::new(None));
#[cfg(feature = "rp2040")]
static SIM_TARGET: Mutex<RefCell<Option<u32>>> = Mutex::new(RefCell::new(None));

#[cfg(feature = "rp2040")]
fn keyboard_sim_start(timer: hal::Timer) {
    let target = timer.get_counter_low().wrapping_add(SIM_BOOT_DELAY_US);
    critical_section::with(|cs| {
        SIM_TIMER.borrow(cs).replace(Some(timer));
        SIM_TARGET.borrow(cs).replace(Some(target));
    });
}

#[cfg(feature = "rp2040")]
fn keyboard_sim_poll_once(cs: critical_section::CriticalSection<'_>) {
    let mut target_slot = SIM_TARGET.borrow(cs).borrow_mut();
    let Some(target) = *target_slot else { return };

    let timer_slot = SIM_TIMER.borrow(cs).borrow();
    let Some(timer) = timer_slot.as_ref() else { return };
    let due = timer.get_counter_low() >= target;
    drop(timer_slot);

    if !due {
        return;
    }
    *target_slot = None;

    let mut queue = KEY_QUEUE.borrow(cs).borrow_mut();
    for c in SIM_KEYSTROKES.chars() {
        queue.push_back(c);
    }
}

#[cfg(not(feature = "rp2040"))]
fn keyboard_sim_poll_once(_cs: critical_section::CriticalSection<'_>) {}
