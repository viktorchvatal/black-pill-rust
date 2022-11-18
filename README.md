# Rust black pill learning demo

My personal walk through learning Rust development on STM32 family of microcontrollers, using:

 - Black Pill development board with STM32F411CEU6 microcontroller as a target device
 - STLink v2 as a programming and debugging interface
 - Debian 10/11 bullseye and Visual studio code as development environment

![Black Pill Photo](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/board/black-pill-board.jpg)

# Other boards

Blue pill board examples
 - https://github.com/viktorchvatal/blue-pill-rust

## Userful Resources

Useful links for programming Black Pill board

 - pin out and board documentation: https://docs.zephyrproject.org/3.2.0/boards/arm/blackpill_f401ce/doc/index.html
 - https://www.jef.land/stm32f411-pit/
 - https://crates.io/crates/stm32f4xx-hal

## Getting Started

All programming devices and software tools are pretty much the same as
for the Blue pill board (check [getting started](https://github.com/viktorchvatal/blue-pill-rust/blob/main/doc/getting_started.md) page for the Blue pill device for details).

Target device specification for the `openocd` tool is `stm32f4x` instead of `stm32f1x`

```
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg
```

Also check different memory limits in the [memory.x](memory.x) file.

## Blinking LED

[Blinking LED example](doc/blinky.md)

![Blinking LED](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/black-blinky/black-blinky-small.gif)

## SH1106 OLED SPI matrix display

[SH1106 display example](doc/display-sh1106.md)

![SH1106 display example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/display-sh1106/display-sh1106-small.gif)

## ADXL345 I2C Accelerometer

[ADXL345 Accelerometer example](doc/accel-adxl345.md)

![ADXL345 Accelerometer example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/accel-adxl345/accel-adxl345-small.gif)

## PCF8563 Real-time clock/calendar [TODO]

[PCF8563 Real-time clock example](doc/time-pcf8563.md)

![PCF8563 Real-time clock example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/time-pcf8563/time-pcf8563-small.gif)

## Reading files from SD card [TODO]

[SD Card reading example](doc/sd-card-read.md)

![SD Card reading example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/sd-card-read/sd-card-read-small.jpg)