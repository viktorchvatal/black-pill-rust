# Rust black pill learning demo

My personal walk through learning Rust development on STM32 family of microcontrollers, using:

 - Black Pill development board with STM32F411CEU6 microcontroller as a target device
 - STLink v2 as a programming and debugging interface
 - Debian 10/11 bullseye and Visual studio code as development environment

# Other boards

Blue pill board examples
 - https://github.com/viktorchvatal/blue-pill-rust

## Userful Resources

Useful links for programming Black Pill board

 - pin out and board documentation: https://docs.zephyrproject.org/3.2.0/boards/arm/blackpill_f401ce/doc/index.html
 - https://www.jef.land/stm32f411-pit/
 - https://crates.io/crates/stm32f4xx-hal

## Getting Started

Run `openocd`

```
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg
```

