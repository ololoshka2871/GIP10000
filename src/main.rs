#![no_main]
#![no_std]

mod support;

mod config;

use panic_abort as _;
use rtic::app;

use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::usb::{Peripheral, UsbBusType};

use stm32f0xx_hal::stm32f0::stm32f0x2::Interrupt as IRQ;

use systick_monotonic::Systick;

use stm32_usbd::UsbBus;
use usb_device::class_prelude::*;
use usb_device::prelude::*;

use usbd_serial::CdcAcmClass;

use support::{IInterruptController, InterruptController};

const CDC_POCKET_SIZE: u16 = 64;

#[app(device = stm32f0xx_hal::pac, peripherals = true, dispatchers = [ADC_COMP])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        #[lock_free]
        ic: InterruptController,
    }

    #[local]
    struct Local {
        usb_device: UsbDevice<'static, UsbBusType>,
        serial: CdcAcmClass<'static, UsbBus<Peripheral>>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        let mut flash = cx.device.FLASH;
        let mut rcc = cx
            .device
            .RCC
            .configure()
            .hsi48()
            .enable_crs(cx.device.CRS)
            .sysclk(48.mhz())
            .freeze(&mut flash);

        let ic = InterruptController::new(cx.core.NVIC);

        let mono = Systick::new(cx.core.SYST, rcc.clocks.sysclk().0);

        let gpioa = cx.device.GPIOA.split(&mut rcc);

        // usb
        let usb = stm32f0xx_hal::usb::Peripheral {
            usb: cx.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        unsafe { USB_BUS.replace(stm32_usbd::UsbBus::new(usb)) };

        let serial = CdcAcmClass::new(
            unsafe { USB_BUS.as_ref().unwrap_unchecked() },
            CDC_POCKET_SIZE,
        );

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap_unchecked() },
            usb_device::prelude::UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("Mksoft")
        .product("gip10000")
        .serial_number("2")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

        (
            Shared { ic },
            Local {
                usb_device: usb_dev,
                serial,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = USB, shared = [ic], local = [usb_device, serial], priority = 1)]
    fn usb_handler(ctx: usb_handler::Context) {
        let usb_device = ctx.local.usb_device;
        let serial = ctx.local.serial;

        // USB dev poll only in the interrupt handler
        if usb_device.poll(&mut [serial]) {
            let mut data = [0u8; CDC_POCKET_SIZE as usize];

            match serial.read_packet(&mut data) {
                Ok(size) if size > 0 => {
                    let _ = serial.write_packet(&data);
                }
                _ => return,
            }
        }

        ctx.shared.ic.unpend(IRQ::USB.into());
    }

    /*
    // next column
    #[task(binds = TIM17)]
    fn tim17_handler(_ctx: tim17_handler::Context) {

    }

    // column writen
    #[task(binds = DMA1_CH2_3)]
    fn dma1_ch2_3_handler(_ctx: dma1_ch2_3_handler::Context) {

    }
    */

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }
}
