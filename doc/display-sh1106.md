# SH1106 OLED SPI matrix display

Example code: [demo-display-sh1106/src/main.rs](../demo/demo-display-sh1106/src/main.rs)

![SH1106 display example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/display-sh1106/display-sh1106.gif)

SH1106 is a very nicely readable OLED (white or blue) display connected
using the SPI interface and controlled by the `sh1106` driver that supports
`embedded_graphics` for rendering.

## Connection

| MCU Board   |     Other          | TM1367 Board | Comment      |
| ----------- | ------------------ | ------------ | ------------ |
| -           |  pull down (GND)   | CS           | Chip select  |
| PB6         |                    | DC           | Data/command |
| PB14        |                    | RES          | Reset        |
| PB15        |                    | MOSI         |              |
| PB13        |                    | CLK          |              |
| -           | VCC                | VCC          |              |
| -           | GND                | GND          |              |

## Notes

Note: connect the display RES pin to the MCU and do not forget to call the
`display.reset` before sending other data/commands, otherwise the picture
on the display would be totally scattered

```rust
display.reset(&mut display_reset, &mut delay).map_err(|_| ())?;
```
