// The two entry points application/app.deor calls into: bring-up, then
// per-panel text output. Both are plain-data (int/string) calls, so
// nothing hardware-shaped ever has to cross the Deor/Rust boundary.

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
