#![no_std]
#![no_main]

mod time;
mod sd_logger;

use arrayvec::{ArrayString};
use pcf8563::PCF8563;
use sd_logger::{append_to_file, SdWriteError};
use time::ClockData;
use core::{fmt::Write, panic::PanicInfo};
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::FONT_6X10}, text::Text
};
use embedded_hal::{spi};
use embedded_sdmmc::{Controller, SdMmcSpi};
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin, i2c::I2c};

/// Turn on onboard LED in case of panic
#[inline(never)]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    let dp = unsafe { pac::Peripherals::steal() };
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();
    let _ = led.set_low();
    loop { }
}

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

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    let dc = gpiob.pb6.into_push_pull_output();

    let spi = dp.SPI2.spi(
        (gpiob.pb13, NoPin, gpiob.pb15),
        spi::MODE_0,
        4000.kHz(),
        &clocks,
    );

    let i2c = I2c::new(
        dp.I2C1,
        (
            gpiob.pb8.into_alternate().set_open_drain(),
            gpiob.pb9.into_alternate().set_open_drain(),
        ),
        400.kHz(),
        &clocks,
    );

    let mut rtc_driver = PCF8563::new(i2c);
    let mut clock = ClockData::default();

    let mut display_reset = gpiob.pb14.into_push_pull_output();
    let mut delay = dp.TIM5.delay_us(&clocks);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .with_size(DisplaySize::Display128x64)
        .connect_spi(spi, dc, sh1106::builder::NoOutputPin::new())
        .into();

    display.reset(&mut display_reset, &mut delay).map_err(|_| ())?;
    display.init().unwrap();

    display_text(&mut display, "Initializing ...").unwrap();

    let sd_spi = dp.SPI1.spi(
        (gpioa.pa5, gpioa.pa6, gpioa.pa7),
        spi::MODE_0,
        400.kHz(),
        &clocks,
    );

    let sd_cs = gpiob.pb0.into_push_pull_output();
    let mut sd_controller = Controller::new(SdMmcSpi::new(sd_spi, sd_cs), ClockData::default());
    let mut counter: usize = 0;

    loop {
        let mut text = ArrayString::<200>::new();

        if let Ok(new_date_time) = rtc_driver.get_datetime() {
            clock.set_from_pcf8563(new_date_time)
        } else {
            clock.reset_to_default();
        }

        write_date_time(&mut text, &clock)?;
        writeln!(&mut text, "").map_err(|_| ())?;

        let mut file_name = ArrayString::<15>::new();
        write_file_name(&mut file_name, &clock)?;

        let mut file_line = ArrayString::<100>::new();
        write_file_line(&mut file_line, &clock, counter)?;

        match append_to_file(&mut sd_controller, &file_name, &file_line) {
            Ok(_) => {
                let _ = writeln!(&mut text, "Line written\n{}\n{}", file_name, file_line);
            },
            Err(error) => {
                let _ = writeln!(&mut text, "SD Write failed\n{:?}", error);

                if let SdWriteError::CannotWriteToOpenedFile(message) = error {
                    let _ = writeln!(&mut text, "{}", message);
                }
            }
        }

        display_text(&mut display, &text)?;
        delay.delay_ms(1000u16);
        counter += 1;
    }
}

fn write_file_line(
    output: &mut dyn Write,
    time: &ClockData,
    counter: usize,
) -> Result<(), ()> {
    writeln!(
        output,
        "{}-{}-{} {}:{:02}:{:02} {} A",
        time.year(), time.month(), time.day(),
        time.hours(), time.minutes(), time.seconds(),
        counter
    ).map_err(|_| ())
}

fn write_file_name(
    output: &mut dyn Write,
    time: &ClockData,
) -> Result<(), ()> {
    write!(
        output,
        "{}{:02}{:02}.log",
        time.year(), time.month(), time.day(),
    ).map_err(|_| ())
}

fn write_date_time(
    output: &mut dyn Write,
    time: &ClockData,
) -> Result<(), ()> {
    write!(
        output,
        "{}.{}.{} {}:{:02}:{:02}",
        time.day(), time.month(), time.year(),
        time.hours(), time.minutes(), time.seconds()
    ).map_err(|_| ())
}

fn display_text<T>(
    display: &mut GraphicsMode<T>,
    message: &str
) -> Result<(), ()>
where T: DisplayInterface {
    display.clear();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let position = Point::new(0, 8);
    Text::new(&message, position, style).draw(display).map_err(|_| ())?;
    display.flush().map_err(|_| ())
}