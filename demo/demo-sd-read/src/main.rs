#![no_std]
#![no_main]

use arrayvec::{ArrayString, ArrayVec};
use core::{fmt::Write};
use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::FONT_6X10}, text::Text
};
use embedded_hal::spi;
use embedded_sdmmc::{
    Controller, SdMmcSpi, TimeSource, Timestamp, VolumeIdx, Volume, Mode, Directory
};
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
    let mut out_text = ArrayString::<100>::new();
    let mut volume: Option<Volume> = open_file_volume(&mut sd_controller, &mut out_text);

    if let Some(ref mut volume) = volume {
        // Store a few first file names just to print their contents later
        let mut file_names: ArrayVec<ArrayString::<12>, 3> = ArrayVec::new();

        let root_dir = match sd_controller.open_root_dir(volume) {
            Ok(dir) => Some(dir),
            Err(_err) => {
                writeln!(&mut out_text, "Root dir read ERR").unwrap();
                None
            }
        };

        if let Some(dir) = root_dir {
            read_file_names(&mut sd_controller, volume, &dir, &mut file_names);
            print_file_contents(&mut sd_controller, volume, &dir, &file_names, &mut out_text);
        }

    }

    display_text(&mut display, &out_text).unwrap();

    loop {

    }
}

/// Print file names and contents (first 32 bytes represented as ASCII test)
/// from the `file_names` vector
fn print_file_contents<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    volume: &mut Volume,
    dir: &Directory,
    file_names: &[ArrayString::<12>],
    out: &mut dyn Write
)
where
    SPI: embedded_hal::spi::FullDuplex<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    T: TimeSource,
    <SPI as embedded_hal::spi::FullDuplex<u8>>::Error: core::fmt::Debug
{
    const READ: Mode = Mode::ReadOnly;

    for file_name in file_names {
        if file_name.len() > 0 {
            if let Ok(mut file) = controller.open_file_in_dir(volume, dir, file_name, READ) {
                let _ = writeln!(out, "* {}", file_name);
                let mut buffer = [0u8; 32];
                let read_size = controller.read(&volume, &mut file, &mut buffer).unwrap();
                let bytes = &buffer[0..read_size];

                for byte in bytes {
                    let _ = write!(out, "{}", *byte as char);
                }

                if read_size > 0 {
                    let _ = writeln!(out, "");
                }
            }
        }
    }
}

/// Read up to first N file names into `file_names` vector
fn read_file_names<SPI, CS, T, const N: usize>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    volume: &Volume,
    dir: &Directory,
    file_names: &mut ArrayVec<ArrayString::<12>, N>
)
where
    SPI: embedded_hal::spi::FullDuplex<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    T: TimeSource,
    <SPI as embedded_hal::spi::FullDuplex<u8>>::Error: core::fmt::Debug
{
    let _ = controller
        .iterate_dir(volume, &dir, |item| {
            let mut file_name = ArrayString::<12>::new();
            let _ = write!(&mut file_name, "{}", item.name);
            if !file_names.is_full() {
                file_names.push(file_name);
            }
        });
}

/// Open first SD card volume and return it
/// Generate debug output into `out` writable
fn open_file_volume<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    out: &mut dyn Write
) -> Option<Volume>
where
    SPI: embedded_hal::spi::FullDuplex<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    T: TimeSource,
    <SPI as embedded_hal::spi::FullDuplex<u8>>::Error: core::fmt::Debug
{
    match controller.device().init() {
        Ok(_) => {
            match controller.device().card_size_bytes() {
                Ok(size) => writeln!(out, "SD OK: {} MB", size >> 20).unwrap(),
                Err(_err) => writeln!(out, "SD Card Connected\nCannot read size").unwrap(),
            }
            match controller.get_volume(VolumeIdx(0)) {
                Ok(volume) => {
                    writeln!(out, "Get FAT Volume 0: OK").unwrap();
                    Some(volume)
                },
                Err(_err) => {
                    writeln!(out, "Vol 0 cannot read FAT").unwrap();
                    None
                },
            }
        }
        Err(_err) => {
            writeln!(out, "SD Card Error\nCannot connect").unwrap();
            None
        },
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