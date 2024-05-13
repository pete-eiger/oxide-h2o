#![no_std]
#![no_main]

extern crate panic_halt;
use arduino_hal::adc::Adc;
use arduino_hal::hal::delay::Delay;
use arduino_hal::hal::clock::MHz16;
use ag_lcd::{Display, Blink, Cursor, LcdDisplay};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let delay = Delay::<MHz16>::new();  // Specifying MHz16 as the clock speed

    // Setup ADC
    let mut adc = Adc::new(dp.ADC, Default::default());
    let analog_pin = pins.a0.into_analog_input(&mut adc);

    let mut red_led = pins.d3.into_output();
    let mut yellow_led = pins.d4.into_output();
    let mut green_led = pins.d5.into_output();

    let rs = pins.d12.into_output().downgrade();
    let en = pins.d10.into_output().downgrade();
    let d4 = pins.d6.into_output().downgrade();
    let d5 = pins.d7.into_output().downgrade();
    let d6 = pins.d8.into_output().downgrade();
    let d7 = pins.d9.into_output().downgrade();

    let mut lcd = LcdDisplay::new(rs, en, delay)
        .with_half_bus(d4, d5, d6, d7)
        .with_display(Display::On)
        .with_blink(Blink::Off)
        .with_cursor(Cursor::Off)
        .build();

    loop {
        let sensor_value = analog_pin.analog_read(&mut adc);
        // Write to serial for debugging
        if let Err(_e) = ufmt::uwriteln!(&mut serial, "Moisture Level: {}\r", sensor_value) {
            // TODO: Handle serial write error
        }

        // Update LEDs based on moisture level
        if sensor_value < 200 {
            red_led.set_high();
            yellow_led.set_low();
            green_led.set_low();
        } else if sensor_value <= 250 {
            red_led.set_low();
            yellow_led.set_high();
            green_led.set_low();
        } else {
            red_led.set_low();
            yellow_led.set_low();
            green_led.set_high();
        }

        // Convert sensor value to string manually
        let mut value_str = [0u8; 10]; // Buffer for up to 10 digits
        let mut temp_value = sensor_value;
        let mut i = 0;
        while temp_value > 0 {
            let digit = temp_value % 10;
            value_str[9 - i] = digit as u8 + b'0'; // Store ASCII character
            temp_value /= 10;
            i += 1;
        }
        if i == 0 { // Handle the case of sensor_value being 0
            value_str[9] = b'0';
            i = 1;
        }

        lcd.clear();
        lcd.print(core::str::from_utf8(&value_str[10 - i..]).unwrap()); // Print the string slice corresponding to the number of digits

        arduino_hal::delay_ms(1000);
    }
}
