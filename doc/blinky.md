# Blinking LED

Example code: [demo-blinky/src/main.rs](../app/demo-blinky/src/main.rs)

![Blinking LED](https://raw.githubusercontent.com/viktorchvatal/black-pill-rust-assets/master/black-blinky/black-blinky.gif)

Blinking LED works the same way as for the Blue pill interface, all platform
agnostic drivers work tha same way, but because Black pill carries a different
microcontroller, `stm32f4xx-hal` hardware abstraction layer is used to
to control all device hardware

```toml
[dependencies.stm32f4xx-hal]
version = "0.13.2"
features = ["stm32f411"]
```

STM32F4 HAL basic initialization is basically as follows:

```rust
#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(100.MHz()).hclk(25.MHz()).freeze();
        let mut delay = dp.TIM5.delay_us(&clocks);
        let gpioc = dp.GPIOC.split();
        let mut led = gpioc.pc13.into_push_pull_output();

        loop {
            led.set_high();
            delay.delay_ms(500u16);
            led.set_low();
            delay.delay_ms(500u16);
        }
    } else {
        loop {}
    }
}
```