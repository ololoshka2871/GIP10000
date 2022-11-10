use core::cell::RefCell;
use core::ops::DerefMut;

use alloc::sync::Arc;

use cortex_m::interrupt::Mutex;
use stm32f4xx_hal::dma::StreamsTuple;

#[allow(unused_imports)]
use stm32f4xx_hal::gpio::{
    Alternate, Analog, Output, Speed, PA0, PA1, PA11, PA12, PA2, PA3, PA5, PA6, PA7, PA8, PB0,
    PB12, PB13, PB3, PB4, PB5, PB6, PB7, PB8, PC10, PC13, PD10, PD11, PD13, PE12,
};

use stm32f4xx_hal::pac::{interrupt, DMA2, GPIOB, SPI1};
use stm32f4xx_hal::spi::NoMiso;
use stm32f4xx_hal::{gpio::PushPull, pac::Interrupt as IRQ, pac::TIM11, prelude::*, time::Hertz};

use crate::parralel_port;
use crate::{
    output::Gip10000llDriver,
    support::{interrupt_controller::IInterruptController, InterruptController},
};

use super::WorkMode;

parralel_port!(Catodes, GPIOB, gpiob::Parts, stm32f4xx_hal::pac::gpiob::RegisterBlock,
    u16 => (pb3, pb4, pb5, pb6, pb7, pb8, pb12, pb13)
);

static DISPLAY: Mutex<
    RefCell<
        Option<
            Gip10000llDriver<
                SPI1,
                (
                    PA5<Alternate<5, PushPull>>,
                    NoMiso,
                    PA7<Alternate<5, PushPull>>,
                ),
                PA1<Output>,
                TIM11,
                DMA2,
                Catodes,
                3,
            >,
        >,
    >,
> = Mutex::new(RefCell::new(None));

#[allow(unused)]
pub struct HighPerformanceMode {
    clocks: stm32f4xx_hal::rcc::Clocks,

    usb_global: stm32f4xx_hal::pac::OTG_FS_GLOBAL,
    usb_device: stm32f4xx_hal::pac::OTG_FS_DEVICE,
    usb_pwrclk: stm32f4xx_hal::pac::OTG_FS_PWRCLK,
    usb_dm: PA11<Alternate<10, PushPull>>,
    usb_dp: PA12<Alternate<10, PushPull>>,
    interrupt_controller: Arc<dyn IInterruptController>,

    led_pin: PC13<Output>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32f4xx_hal::pac::Peripherals) -> Self {
        let rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        //let dma_channels = dp.DMA1;

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // https://docs.rs/crate/stm32f4xx-hal/0.13.2/source/examples/spi_dma.rs
        // let spi2 = Spi::new(dp.SPI2, (pb13, NoMiso {}, pb15), mode, 3.MHz(), &clocks);
        // https://docs.rs/crate/stm32f4xx-hal/0.13.2/source/examples/i2s-audio-out-dma.rs
        // https://docs.rs/crate/stm32f4xx-hal/0.13.2/source/examples/stopwatch-with-ssd1306-and-interrupts.rs

        let clocks = rcc
            .cfgr
            .use_hse(Hertz::Hz(crate::config::XTAL_FREQ))
            .require_pll48clk()
            .sysclk(Hertz::Hz(crate::config::FREERTOS_CONFIG_FREQ))
            .freeze();

        let mut timer = dp.TIM11.counter(&clocks);
        timer.listen(stm32f4xx_hal::timer::Event::Update);

        ic.set_priority(
            IRQ::TIM1_TRG_COM_TIM11.into(),
            crate::config::UPDATE_COUNTER_INTERRUPT_PRIO,
        );
        ic.unpend(IRQ::TIM1_TRG_COM_TIM11.into());
        ic.unmask(IRQ::TIM1_TRG_COM_TIM11.into());

        let spi1 = dp.SPI1.spi(
            (
                gpioa.pa5.into_alternate(),
                NoMiso {},
                gpioa.pa7.into_alternate(),
            ),
            stm32f4xx_hal::spi::Mode {
                polarity: stm32f4xx_hal::spi::Polarity::IdleLow,
                phase: stm32f4xx_hal::spi::Phase::CaptureOnFirstTransition,
            },
            8.MHz(),
            &clocks,
        );
        /*
        spi1.listen(stm32f4xx_hal::spi::Event::Txe);
        ic.set_priority(
            IRQ::SPI1.into(),
            crate::config::UPDATE_COUNTER_INTERRUPT_PRIO,
        );
        ic.unpend(IRQ::SPI1.into());
        ic.unmask(IRQ::SPI1.into());
        */

        let spi1_dma = StreamsTuple::new(dp.DMA2).3; // SPI1_TX
        ic.set_priority(
            IRQ::DMA2_STREAM3.into(),
            crate::config::UPDATE_COUNTER_INTERRUPT_PRIO,
        );
        ic.unpend(IRQ::DMA2_STREAM3.into());
        ic.unmask(IRQ::DMA2_STREAM3.into());

        let gip10000 = Gip10000llDriver::new(
            timer,
            spi1,
            gpioa
                .pa1
                .into_push_pull_output_in_state(stm32f4xx_hal::gpio::PinState::High),
            spi1_dma,
            Catodes::init(dp.GPIOB), //ParalelBus::new(dp.GPIOB.split(), (3, 4, 5, 6, 7, 8, 12, 13)),
            crate::output::Offsets {
                oe1: Catodes::get_mask_for_pin(12),
                oe2: Catodes::get_mask_for_pin(13),
                a: Catodes::get_mask_for_pin(3),
                b: Catodes::get_mask_for_pin(4),
                c: Catodes::get_mask_for_pin(5),
                d: Catodes::get_mask_for_pin(6),
                e: Catodes::get_mask_for_pin(7),
                f: Catodes::get_mask_for_pin(8),
            },
        );

        cortex_m::interrupt::free(|cs| {
            DISPLAY.borrow(cs).replace(Some(gip10000));
        });

        HighPerformanceMode {
            usb_global: dp.OTG_FS_GLOBAL,
            usb_device: dp.OTG_FS_DEVICE,
            usb_pwrclk: dp.OTG_FS_PWRCLK,

            clocks,

            interrupt_controller: ic,

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
            "1",
            crate::config::USBD_TASK_STACK_SIZE,
            TaskPriority(crate::config::USBD_TASK_PRIO),
        );

        // --------------------------------------------------------------------

        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut disp) = DISPLAY.borrow(cs).borrow_mut().deref_mut() {
                disp.start();
            }
        });

        crate::workmodes::common::create_monitor(sys_clk)?;

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks);
    }
}

#[cfg(feature = "stm32f401")]
#[interrupt]
unsafe fn TIM1_TRG_COM_TIM11() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut disp) = DISPLAY.borrow(cs).borrow_mut().deref_mut() {
            disp.on_timer();
        }
    })
}

/*
#[cfg(feature = "stm32f401")]
#[interrupt]
unsafe fn SPI1() {
    crate::support::led::led_toggle();

    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut disp) = DISPLAY.borrow(cs).borrow_mut().deref_mut() {
            disp.on_spi_done();
        }
    })
}
*/

#[cfg(feature = "stm32f401")]
#[interrupt]
unsafe fn DMA2_STREAM3() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut disp) = DISPLAY.borrow(cs).borrow_mut().deref_mut() {
            disp.on_spi_done();
        }
    })
}
