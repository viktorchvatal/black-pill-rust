# ADXL345 I2C Accelerometer

Example code: [demo-accel-adxl345/src/main.rs](../demo/demo-accel-adxl345/src/main.rs)

![ADXL345 Accelerometer example](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/accel-adxl345/accel-adxl345.gif)

ADXL345 is an I2C accelerometer that can be interfaced using the `adxl343` crate

## Connection

| MCU Board   |     Other          | TM1367 Board |
| ----------- | ------------------ | ------------ |
| PB8         | pull up 5K         | SCL          |
| PB9         | pull up 5K         | SDA          |
| -           | VCC                | VCC          |
| -           | GND                | GND          |

## Notes

Note: i was getting really strange readings for X, Y, Z acceleration with
default 2G range (both RANGE_HI and RANGE_LO bit unset), but the accelerometer
or library seems to work well with any of the 4G, 8G, or 16G ranges

I prefer 8G range (there is more room for values larger than 1G), that can
be set uring the data format

```rust
let format: DataFormatFlags = DataFormatFlags::RANGE_HI;
let mut accelerometer = Adxl343::new_with_data_format(i2c, format).unwrap();
```
