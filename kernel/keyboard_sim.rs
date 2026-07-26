// Wokwi/rp2040 stand-in only. Wokwi can't simulate a real PS/2 keyboard,
// so a hardware timer alarm fires a few seconds after boot and feeds a
// canned string into the same KEY_QUEUE the real PS/2 receiver
// (keyboard.rs) fills — app-level code draining xk_read_key() can't tell
// the two apart. Compiled out entirely on real hardware (feature =
// "rp2350"): kb_read_key() there only ever sees genuine keystrokes.

#[cfg(feature = "rp2040")]
use hal::timer::Alarm;

#[cfg(feature = "rp2040")]
const SIM_KEYSTROKES: &str = "hello world\n";
#[cfg(feature = "rp2040")]
const SIM_BOOT_DELAY_US: u32 = 5_000_000;
#[cfg(feature = "rp2040")]
const SIM_KEY_GAP_US: u32 = 120_000;

#[cfg(feature = "rp2040")]
static SIM_ALARM: Mutex<RefCell<Option<hal::timer::Alarm0>>> = Mutex::new(RefCell::new(None));
#[cfg(feature = "rp2040")]
static SIM_INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));

#[cfg(feature = "rp2040")]
fn keyboard_sim_start(mut timer: hal::Timer) {
    let mut alarm = timer.alarm_0().unwrap();
    alarm.schedule(hal::fugit::MicrosDurationU32::micros(SIM_BOOT_DELAY_US)).unwrap();
    alarm.enable_interrupt();

    critical_section::with(|cs| {
        SIM_ALARM.borrow(cs).replace(Some(alarm));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::TIMER_IRQ_0);
    }
}

#[cfg(feature = "rp2040")]
#[interrupt]
fn TIMER_IRQ_0() {
    critical_section::with(|cs| {
        let mut alarm_slot = SIM_ALARM.borrow(cs).borrow_mut();
        let Some(alarm) = alarm_slot.as_mut() else { return };
        alarm.clear_interrupt();

        let mut idx = SIM_INDEX.borrow(cs).borrow_mut();
        let bytes = SIM_KEYSTROKES.as_bytes();
        if *idx < bytes.len() {
            KEY_QUEUE.borrow(cs).borrow_mut().push_back(bytes[*idx] as char);
            *idx += 1;
        }

        if *idx < bytes.len() {
            let _ = alarm.schedule(hal::fugit::MicrosDurationU32::micros(SIM_KEY_GAP_US));
        }
    });
}
