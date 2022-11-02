use alloc::sync::Arc;

#[allow(unused_imports)]
use stm32f4xx_hal::gpio::{
    Alternate, Analog, Output, Speed, PA0, PA1, PA11, PA12, PA2, PA3, PA6, PA7, PA8, PB0, PC10,
    PD10, PD11, PD13, PE12,
};
use stm32f4xx_hal::{gpio::PushPull, prelude::*, time::Hertz};

use crate::support::{interrupt_controller::IInterruptController, InterruptController};

use super::WorkMode;

#[allow(unused)]
pub struct HighPerformanceMode {
    clocks: stm32f4xx_hal::rcc::Clocks,

    usb_global: stm32f4xx_hal::pac::OTG_FS_GLOBAL,
    usb_device: stm32f4xx_hal::pac::OTG_FS_DEVICE,
    usb_pwrclk: stm32f4xx_hal::pac::OTG_FS_PWRCLK,
    usb_dm: PA11<Alternate<10, PushPull>>,
    usb_dp: PA12<Alternate<10, PushPull>>,
    interrupt_controller: Arc<dyn IInterruptController>,

    display_timer: stm32f4xx_hal::pac::TIM11,

    led_pin: stm32f4xx_hal::gpio::Pin<'C', 13, Output>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32f4xx_hal::pac::Peripherals) -> Self {
        let rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        //let dma_channels = dp.DMA1;

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        HighPerformanceMode {
            usb_global: dp.OTG_FS_GLOBAL,
            usb_device: dp.OTG_FS_DEVICE,
            usb_pwrclk: dp.OTG_FS_PWRCLK,

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

            usb_dm: gpioa.pa11.into_alternate(),
            usb_dp: gpioa.pa12.into_alternate(),
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
        use crate::threads::usbd::{Usbd, UsbdPeriph};
        use freertos_rust::TaskPriority;

        let sys_clk = self.clocks.hclk();

        crate::support::led::led_init(self.led_pin);

        {
            defmt::trace!("Creating usb thread...");
            let usbperith = UsbdPeriph {
                usb_global: self.usb_global,
                usb_device: self.usb_device,
                usb_pwrclk: self.usb_pwrclk,
                pin_dm: self.usb_dm,
                pin_dp: self.usb_dp,
                hclk: sys_clk,
            };
            let ic = self.interrupt_controller.clone();
            Usbd::init(usbperith, ic, crate::config::USB_INTERRUPT_PRIO);
        }

        {
            let serial = Usbd::serial_port();
            let data_input_server = {
                defmt::trace!("Creating G-Code server thread...");
                freertos_rust::Task::new()
                    .name("G-Code")
                    .stack_size(
                        (crate::config::G_CODE_TASK_STACK_SIZE / core::mem::size_of::<u32>())
                            as u16,
                    )
                    .priority(TaskPriority(crate::config::GCODE_TASK_PRIO))
                    .start(move |_| {
                        crate::threads::data_input_server::gcode_server(
                            serial, /*, gcode_queue, req_queue*/
                        )
                    })
                    .expect("expect5")
            };
            Usbd::subscribe(data_input_server);
        }

        // --------------------------------------------------------------------

        let _ = Usbd::start(
            usb_device::prelude::UsbVidPid(0x0483, 0x573E),
            "gip10000",
            "MKsoft",
            "0",
            crate::config::USBD_TASK_STACK_SIZE,
            TaskPriority(crate::config::USBD_TASK_PRIO),
        );

        // --------------------------------------------------------------------

        crate::workmodes::common::create_monitor(sys_clk)?;

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks);
    }
}
