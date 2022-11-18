#![no_std]
#![no_main]

use arrayvec::ArrayString;
use core::{fmt::Write};
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::FONT_6X10}, text::Text
};
use embedded_hal::spi;
use embedded_sdmmc::{Controller, SdMmcSpi, TimeSource, Timestamp, VolumeIdx};
use panic_halt as _;
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin};

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

    let mut sd_controller = Controller::new(SdMmcSpi::new(sd_spi, sd_cs), Clock {});
    let mut out = ArrayString::<100>::new();

    match sd_controller.device().init() {
        Ok(_) => {
            match sd_controller.device().card_size_bytes() {
                Ok(size) => write!(&mut out, "SD OK: {} MB\n", size >> 20).unwrap(),
                Err(_err) => write!(&mut out, "SD Card Connected\nCannot read size\n").unwrap(),
            }
            match sd_controller.get_volume(VolumeIdx(0)) {
                Ok(_fs) => write!(out, "Get FAT Volume 0: OK").unwrap(),
                Err(_err) => write!(out, "Vol 0 cannot read FAT").unwrap(),
            }
        }
        Err(_err) => write!(&mut out, "SD Card Error\nCannot connect").unwrap(),
    }

    display_text(&mut display, &out).unwrap();

    loop {

    }
}

struct Clock;

impl TimeSource for Clock {
    // Fake time source that just returns 1. 1. 1970 0:00:00
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
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