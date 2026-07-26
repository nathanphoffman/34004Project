// Bit-banged HD44780 4-bit driver, shared by all three panels via the
// types in state.rs (DataBus/Lcd).

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
