//! Blinks an LED and outputs ON and OFF debug messages via semihosting i/o
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.
//!
//! Original source: https://github.com/stm32-rs/stm32f1xx-hal/blob/master/examples/blinky.rs

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{pac, prelude::*};
use embedded_hal::{digital::v2::OutputPin, blocking::delay::{DelayUs, DelayMs}};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();

        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(100.MHz()).hclk(25.MHz()).freeze();

        // Create a delay abstraction based on general-pupose 32-bit timer TIM5
        let mut delay = dp.TIM5.delay_us(&clocks);

        // Acquire the GPIOC peripheral
        let gpioc = dp.GPIOC.split();

        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_high();

        let message = ".... . .-.. .-.. ---    ";

        // Transmit the message using the onboard LED
        loop {
            transmit_message(&mut led, &mut delay, &message).unwrap();
        }
    } else {
        loop {}
    }

}

fn transmit_message<LED, D>(
    led: &mut LED,
    delay: &mut D,
    message: &str
) -> Result<(),()>
where LED: OutputPin, D: DelayUs<u16> + DelayMs<u16>, {
    for char in message.chars() {
        match char {
            // Just delay
            ' ' => delay.delay_ms(500),
            // Short blink
            '.' => {
                led.set_low().map_err(|_| ())?;
                delay.delay_ms(200);
                led.set_high().map_err(|_| ())?;
            },
            // Long blink
            '-' => {
                led.set_low().map_err(|_| ())?;
                delay.delay_ms(700);
                led.set_high().map_err(|_| ())?;
            },
            _other => {},
        }
        delay.delay_ms(300);
    }

    Ok(())
}
