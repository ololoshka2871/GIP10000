#![no_main]
#![no_std]

use panic_abort as _;
use rtic::app;

use stm32f0xx_hal::gpio::{gpioa::PA0, Output, PushPull};
use stm32f0xx_hal::prelude::*;
use systick_monotonic::{fugit::Duration, Systick};

#[app(device = stm32f0xx_hal::pac, peripherals = true, dispatchers = [ADC_COMP])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PA0<Output<PushPull>>,
        state: bool,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let (led, mono) = cortex_m::interrupt::free(move |cs| {
            // Setup clocks
            let mut flash = cx.device.FLASH;
            let mut rcc = cx
                .device
                .RCC
                .configure()
                .hsi48()
                .enable_crs(cx.device.CRS)
                .sysclk(48.mhz())
                .freeze(&mut flash);

            let mono = Systick::new(cx.core.SYST, rcc.clocks.sysclk().0);

            // Setup LED
            let gpioa = cx.device.GPIOA.split(&mut rcc);
            let led = gpioa.pa0.into_push_pull_output(cs);

            // Schedule the blinking task
            blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

            (led, mono)
        });

        (
            Shared {},
            Local { led, state: false },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, state])]
    fn blink(cx: blink::Context) {
        if *cx.local.state {
            let _ = cx.local.led.set_high();
            *cx.local.state = false;
        } else {
            let _ = cx.local.led.set_low();
            *cx.local.state = true;
        }
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }
}
