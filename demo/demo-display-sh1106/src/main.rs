#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Circle}
};
use embedded_hal::spi;
use panic_halt as _;
use sh1106::{prelude::*, Builder};
use stm32f4xx_hal::{prelude::*, pac, spi::{Spi, NoMiso}, gpio::NoPin};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        run(dp, cp);
    } else {
        loop {}
    }
}

fn run(
    dp: pac::Peripherals,
    _cp: cortex_m::Peripherals,
) -> ! {
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
    let mut led = gpioc.pc13.into_push_pull_output();
    let cs = sh1106::builder::NoOutputPin::new();

    let mut delay = dp.TIM5.delay_us(&clocks);

    // If you aren't using the Chip Select pin, use this instead:
    // let mut display: GraphicsMode<_> = Builder::new()
    //     .connect_spi(spi, dc, sh1106::builder::NoOutputPin::new())
    //     .into();

    display_reset.set_low();
    delay.delay_ms(50u16);
    display_reset.set_high();

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .with_size(DisplaySize::Display128x64)
        .connect_spi(spi, dc, cs)
        .into();

    display.init().unwrap();
    display.set_contrast(100);

    let mut size_1 = 80;
    let mut size_2 = 0;

    loop {
        led.set_low();

        display.clear();

        let _result = Circle::new(Point::new(0, 0), size_1)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 3))
            .draw(&mut display);

        let _result = Circle::new(Point::new(0, 0), size_2)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 3))
            .draw(&mut display);

        size_1 = (size_1 + 1) % 160;
        size_2 = (size_2 + 1) % 160;

        display.flush().unwrap();
        led.set_high();

        delay.delay_ms(20u16);
    }
}