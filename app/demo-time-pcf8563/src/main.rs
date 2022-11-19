#![no_std]
#![no_main]

use core::{fmt::Write};
use arrayvec::ArrayString;
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::FONT_10X20}, text::Text
};
use embedded_hal::spi;
use panic_halt as _;
use pcf8563::{PCF8563, DateTime};
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin, i2c::I2c};

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
    _cp: cortex_m::Peripherals,
) -> Result<(), ()> {
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(100.MHz()).hclk(25.MHz()).freeze();

    let gpiob = dp.GPIOB.split();

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

    let i2c = I2c::new(
        dp.I2C1,
        (
            gpiob.pb8.into_alternate().set_open_drain(),
            gpiob.pb9.into_alternate().set_open_drain(),
        ),
        400.kHz(),
        &clocks,
    );

    display_text(&mut display, "Starting up...").unwrap();

    let mut rtc = PCF8563::new(i2c);

    /*
    // Set date and time
    let now = DateTime {
        year: 22,
        month: 11,
        weekday: 5,
        day: 18,
        hours: 11,
        minutes: 28,
        seconds: 00,
    };

    rtc.set_datetime(&now).unwrap();
     */

    loop {
        display.clear();

        let mut text = ArrayString::<40>::new();

        match rtc.get_datetime() {
            Ok(datetime) => {
                render_date(&mut text, datetime).unwrap();
            },
            Err(error) => {
                let _ = write!(&mut text, "Error:\n{:?}", error);
            }
        };

        display_text(&mut display, &text).unwrap();

        delay.delay_ms(100u16);
    }
}

const DAYS: &[&str] = &[
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

fn render_date<W>(
    destination: &mut W,
    datetime: DateTime
) -> Result<(), ()> where W: Write {
    write!(
        destination,
        "{:02}.{:02}.20{:02}\n{:02}:{:02}:{:02}\n{}",
        datetime.day,
        datetime.month,
        datetime.year,
        datetime.hours,
        datetime.minutes,
        datetime.seconds,
        DAYS[datetime.weekday as usize]
    ).map_err(|_| ())
}

fn display_text<T>(
    display: &mut GraphicsMode<T>,
    message: &str
) -> Result<(), ()>
where T: DisplayInterface {
    display.clear();

    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let position = Point::new(0, 14);
    Text::new(&message, position, style).draw(display).map_err(|_| ())?;
    display.flush().map_err(|_| ())
}