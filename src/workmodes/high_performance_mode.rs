use alloc::sync::Arc;

#[allow(unused_imports)]
use stm32f4xx_hal::gpio::{
    Alternate, Analog, Output, PushPull, Speed, PA0, PA1, PA11, PA12, PA2, PA3, PA6, PA7, PA8, PB0,
    PC10, PD10, PD11, PD13, PE12,
};
use stm32f4xx_hal::{prelude::*, time::Hertz};

use crate::support::{interrupt_controller::IInterruptController, InterruptController};

use super::WorkMode;

#[allow(unused)]
pub struct HighPerformanceMode {
    clocks: stm32f4xx_hal::rcc::Clocks,

    usb_global: stm32f4xx_hal::pac::OTG_FS_GLOBAL,
    usb_device: stm32f4xx_hal::pac::OTG_FS_DEVICE,
    usb_pwrclk: stm32f4xx_hal::pac::OTG_FS_PWRCLK,
    //usb_dm: PA11<Alternate<PushPull, 10>>,
    //usb_dp: PA12<Alternate<PushPull, 10>>,
    interrupt_controller: Arc<dyn IInterruptController>,

    display_timer: stm32f4xx_hal::pac::TIM11,

    led_pin: stm32f4xx_hal::gpio::Pin<'C', 13, Output>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32f4xx_hal::pac::Peripherals) -> Self {
        let rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        //let dma_channels = dp.DMA1;

        let mut gpioa = dp.GPIOA.split();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();

        HighPerformanceMode {
            usb_global: dp.OTG_FS_GLOBAL,
            usb_device: dp.OTG_FS_DEVICE,
            usb_pwrclk: dp.OTG_FS_PWRCLK,

            //usb_dm: gpioa.pa11.into_alternate().set_speed(Speed::VeryHigh),
            //usb_dp: gpioa.pa12.into_alternate().set_speed(Speed::VeryHigh),
            clocks: rcc
                .cfgr
                .use_hse(Hertz::Hz(crate::config::XTAL_FREQ))
                .require_pll48clk()
                .sysclk(Hertz::Hz(crate::config::FREERTOS_CONFIG_FREQ))
                .freeze(),

            interrupt_controller: ic,
            display_timer: dp.TIM11,

            led_pin: gpioc
                .pc13
                .into_push_pull_output_in_state(stm32f4xx_hal::gpio::PinState::High),
        }
    }

    fn configure_clock(&mut self) {
        crate::time_base::master_counter::MasterCounter::init(
            if self.clocks.ppre2() > 1 {
                self.clocks.pclk2()
            } else {
                self.clocks.pclk2() * 2
            },
            self.interrupt_controller.clone(),
        );
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        let sys_clk = self.clocks.hclk();

        crate::support::led::led_init(self.led_pin);

        /*
        {
            use crate::threads::usbd::{Usbd, UsbdPeriph};
            defmt::trace!("Creating usb thread...");
            let usbperith = UsbdPeriph {
                usb: self.usb,
                pin_dp: self.usb_dp,
                pin_dm: self.usb_dm,
            };
            let ic = self.interrupt_controller.clone();
            Usbd::init(usbperith, ic, crate::config::USB_INTERRUPT_PRIO);
        }
        */

        crate::workmodes::common::create_monitor(sys_clk)?;

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks);
    }
}
