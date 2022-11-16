#![no_std]
#![no_main]

use core::fmt::Write;
use arrayvec::ArrayString;
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::{FONT_5X8, FONT_7X13_BOLD}}, text::Text
};
use embedded_hal::spi;
use panic_halt as _;
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin, i2c::I2c};
use adxl343::{Adxl343, accelerometer::Accelerometer};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        run(dp, cp).unwrap();
        loop {}
    } else {
        loop {}
    }
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

    let dc = gpiob.pb6.into_push_pull_output();

    let spi = dp.SPI2.spi(
        (gpiob.pb13, NoPin, gpiob.pb15),
        spi::MODE_0,
        4000.kHz(),
        &clocks,
    );

    let mut display_reset = gpiob.pb14.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .with_size(DisplaySize::Display128x64)
        .connect_spi(spi, dc, sh1106::builder::NoOutputPin::new())
        .into();

    display.reset(&mut display_reset, &mut delay).map_err(|_| ())?;
    display.init().unwrap();

    let mut i2c = I2c::new(
        dp.I2C1,
        (
            gpiob.pb8.into_alternate().set_open_drain(),
            gpiob.pb9.into_alternate().set_open_drain(),
        ),
        400.kHz(),
        &clocks,
    );

    let mut accelerometer = match Adxl343::new(i2c) {
        Ok(device) => device,
        Err(_error) => stop_on_error(
            display, "Accelerometer demo\nFailed to\ninitialize"
        )
    };

    loop {
        display.clear();
        match accelerometer.accel_norm() {
            Ok(values) => {
                let mut text = ArrayString::<100>::new();
                let _ = write!(
                    &mut text,
                    "Accelerometer demo\nX = {}\nY = {}\nZ = {}",
                    values.x, values.y, values.z
                );
                let _ = print(&mut display, &text);
            },
            Err(_error) => {
                let _ = print(&mut display, "Read error");
            },
        };

        display.flush().unwrap();
    }
}

fn print<T>(
    display: &mut GraphicsMode<T>,
    message: &str
) -> Result<(), ()>
where T: DisplayInterface{
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    let position = Point::new(0, 8);
    Text::new(&message, position, style).draw(display).map_err(|_| ())?;
    Ok(())
}

fn stop_on_error<T>(
    mut display: GraphicsMode<T>,
    message: &str
) -> !
where T: DisplayInterface{
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    let position = Point::new(0, 8);
    let _ = Text::new(&message, position, style).draw(&mut display);
    let _ = display.flush();
    loop {}
}