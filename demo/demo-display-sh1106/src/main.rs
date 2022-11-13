#![no_std]
#![no_main]

use core::fmt::Write;
use arrayvec::ArrayString;
use cortex_m::peripheral::DWT;
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Circle}, mono_font::{MonoTextStyle, ascii::FONT_7X13_BOLD}, text::Text
};
use embedded_hal::spi;
use panic_halt as _;
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin, time::Hertz};

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
    let mut led = gpioc.pc13.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .with_size(DisplaySize::Display128x64)
        .connect_spi(spi, dc, sh1106::builder::NoOutputPin::new())
        .into();

    display.reset(&mut display_reset, &mut delay).map_err(|_| ())?;
    display.init().unwrap();

    let mut size_1 = 80;
    let mut size_2 = 0;
    let mut duration = 0u32;

    loop {
        let start = DWT::cycle_count();
        display.clear();

        let origin = Point::new(1, 1);
        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 3);

        let _result = Circle::new(origin, size_1).into_styled(style).draw(&mut display);
        let _result = Circle::new(origin, size_2).into_styled(style).draw(&mut display);

        size_1 = (size_1 + 1) % 160;
        size_2 = (size_2 + 1) % 160;

        print_timing(&mut display, duration, clocks.hclk())?;

        led.set_high();
        display.flush().unwrap();
        led.set_low();

        duration = DWT::cycle_count().wrapping_sub(start);
    }
}

fn print_timing<T>(
    display: &mut GraphicsMode<T>,
    duration_cycles: u32,
    sysclk: Hertz
) -> Result<(), ()>
where T: DisplayInterface{
    if duration_cycles == 0 {
        return Ok(())
    }

    let mut text = ArrayString::<20>::new();
    let fps10 = sysclk.to_Hz()*10/duration_cycles;
    let _ = write!(&mut text, "{}.{} FPS", fps10/10, fps10 % 10);
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    let position = Point::new(128 - (text.len() - 1) as i32*8, 63);
    Text::new(&text, position, style).draw(display).map_err(|_| ())?;
    Ok(())
}