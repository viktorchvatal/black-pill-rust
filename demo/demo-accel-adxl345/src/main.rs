#![no_std]
#![no_main]

use cortex_m_rt::{entry};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{MonoTextStyle, ascii::{FONT_7X13_BOLD}},
    text::Text,
    primitives::{PrimitiveStyle, Rectangle, Line}
};
use embedded_hal::spi;
use panic_halt as _;
use sh1106::{prelude::*, Builder, interface::DisplayInterface};
use stm32f4xx_hal::{prelude::*, pac, gpio::NoPin, i2c::I2c};
use adxl343::{Adxl343, accelerometer::{RawAccelerometer, vector::{I16x3}}, DataFormatFlags};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        run(dp, cp).unwrap();
    }

    panic!()
}

fn run(
    dp: pac::Peripherals,
    mut _cp: cortex_m::Peripherals,
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
    display.init().map_err(|_| ())?;

    let i2c = I2c::new(
        dp.I2C1,
        (
            gpiob.pb8.into_alternate().set_open_drain(),
            gpiob.pb9.into_alternate().set_open_drain(),
        ),
        400.kHz(),
        &clocks,
    );

    let format: DataFormatFlags = DataFormatFlags::RANGE_HI;

    let mut accelerometer = match Adxl343::new_with_data_format(i2c, format) {
        Ok(device) => device,
        Err(_error) => stop_on_error(
            display, "Accelerometer demo\nFailed to\ninitialize"
        )
    };

    loop {
        display.clear();

        let values = match accelerometer.accel_raw() {
            Ok(raw_values) => raw_values,
            Err(_error) => {
                let _ = print(&mut display, "Read error");
                I16x3::new(0, 0, 0)
            },
        };

        render_values(&mut display, values)?;
        display.flush().map_err(|_| {})?;
    }
}

const BAR_TOP: i32 = 22;
const BAR_SPACE: i32 = 20;
const BAR_SIZE: Size = Size::new(16, 40);
const CONVERT_G: i32 = 16384;

fn render_values<T>(
    display: &mut GraphicsMode<T>,
    values: I16x3,
) -> Result<(), ()>
where T: DisplayInterface {
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    Text::new("Accelerometer demo", Point::new(0, 8), style).draw(display).map_err(|_| ())?;
    render_bar(display, "X", values.x as i32, 0)?;
    render_bar(display, "Y", values.y as i32, 40)?;
    render_bar(display, "Z", values.z as i32, 80)?;
    render_center_line(display)
}

fn render_bar<T>(
    display: &mut GraphicsMode<T>,
    name: &str,
    value: i32,
    position: i32
) -> Result<(), ()>
where T: DisplayInterface {
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    Text::new(&name, Point::new(position + 5, BAR_TOP), style).draw(display).map_err(|_| ())?;
    let outline_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let filled_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let rect_left = position + BAR_SPACE;
    let center_height = BAR_TOP + BAR_SIZE.height as i32/2;

    Rectangle::new(Point::new(rect_left, BAR_TOP), BAR_SIZE)
        .into_styled(outline_style)
        .draw(display)
        .map_err(|_| ())?;

    let highlight_height: i32 = BAR_SIZE.height as i32/2*value.abs()/CONVERT_G;

    let (rect_position, rect_size) = if value < 0 {
        let position = Point::new(rect_left, center_height - highlight_height);
        let size = Size::new(BAR_SIZE.width, highlight_height as u32);
        (position, size)
    } else {
        let position = Point::new(rect_left, center_height);
        let size = Size::new(BAR_SIZE.width, highlight_height as u32);
        (position, size)
    };

    Rectangle::new(rect_position, rect_size)
        .into_styled(filled_style)
        .draw(display)
        .map_err(|_| ())?;

    Ok(())
}

fn render_center_line<T>(
    display: &mut GraphicsMode<T>,
) -> Result<(), ()>
where T: DisplayInterface {
    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let position: i32 = BAR_TOP + BAR_SIZE.height as i32/2;

    Line::new(Point::new(0, position), Point::new(127, position))
        .into_styled(style)
        .draw(display)
        .map_err(|_| ())
}

fn print<T>(
    display: &mut GraphicsMode<T>,
    message: &str
) -> Result<(), ()>
where T: DisplayInterface {
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    let position = Point::new(0, 8);
    Text::new(&message, position, style).draw(display).map_err(|_| ())?;
    Ok(())
}

fn stop_on_error<T>(
    mut display: GraphicsMode<T>,
    message: &str
) -> !
where T: DisplayInterface {
    let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
    let position = Point::new(0, 8);
    let _ = Text::new(&message, position, style).draw(&mut display);
    let _ = display.flush();
    loop {}
}