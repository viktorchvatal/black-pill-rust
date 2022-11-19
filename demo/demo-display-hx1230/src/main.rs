#![no_std]
#![no_main]

use cortex_m_rt::{entry};
use cortex_m::peripheral::Peripherals as CortexPeripherals;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::{FONT_9X18_BOLD}}, text::{Text, Alignment}
};
use embedded_hal::spi;
use hx1230::{ArrayDisplayBuffer, SpiDriver, DisplayBuffer, DisplayDriver};
use panic_halt as _;
use stm32f4xx_hal::{prelude::*, pac::{self, Peripherals}, gpio::NoPin};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (Peripherals::take(), CortexPeripherals::take()) {
        run(dp, cp).unwrap();
    }

    loop {}
}

fn run(
    dp: pac::Peripherals,
    mut cp: cortex_m::Peripherals,
) -> Result<(), ()> {
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(100.MHz()).hclk(25.MHz()).freeze();

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let mut display_cs = gpiob.pb14.into_push_pull_output();

    let mut spi = dp.SPI2.spi(
        (gpiob.pb13, NoPin, gpiob.pb15),
        spi::MODE_0,
        4000.kHz(),
        &clocks,
    );

    let mut led = gpioc.pc13.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    let mut frame_buffer: ArrayDisplayBuffer = ArrayDisplayBuffer::new();

    let mut display = SpiDriver::new(&mut spi, &mut display_cs);
    display.initialize(&mut delay).map_err(|_| ())?;

    let mut counter: usize = 0;
    let text_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::Off);

    const TEXT1: &str = "HX1230\non\nBlack Pill";
    const TEXT2: &str = "HX1230\non";

    loop {
        led.set_low();
        frame_buffer.clear_buffer(0xff);

        let text = if (counter % 2) == 0 { TEXT1 } else { TEXT2 };

        Text::with_alignment(text, Point::new(48, 20), text_style, Alignment::Center)
            .draw(&mut frame_buffer)
            .map_err(|_| ())?;

        let mut driver = SpiDriver::new(&mut spi, &mut display_cs);
        driver.send_buffer(&frame_buffer).map_err(|_| ())?;

        counter = counter + 1;

        led.set_high();
        delay.delay_ms(1000_u16);
    }
}
