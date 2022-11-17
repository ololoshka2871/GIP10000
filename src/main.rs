#![no_main]
#![no_std]

use panic_abort as _;
use rtic::app;

use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::usb::{Peripheral, UsbBusType};

use systick_monotonic::Systick;

use stm32_usbd::UsbBus;
use usb_device::class_prelude::*;
use usb_device::prelude::*;

use usbd_serial::SerialPort;

#[app(device = stm32f0xx_hal::pac, peripherals = true, dispatchers = [ADC_COMP])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_device: UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBus<Peripheral>>,
    }

    #[local]
    struct Local {}

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

        let mono = Systick::new(cx.core.SYST, rcc.clocks.sysclk().0);

        let gpioa = cx.device.GPIOA.split(&mut rcc);

        // usb
        let usb = stm32f0xx_hal::usb::Peripheral {
            usb: cx.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        unsafe { USB_BUS = Some(stm32_usbd::UsbBus::new(usb)) };

        let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap_unchecked() },
            usb_device::prelude::UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("Mksoft")
        .product("gip10000")
        .serial_number("0")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

        (
            Shared {
                usb_device: usb_dev,
                serial,
            },
            Local {},
            init::Monotonics(mono),
        )
    }

    #[task(binds = USB, shared = [usb_device, serial])]
    fn usb_handler(ctx: usb_handler::Context) {
        let usb_device = ctx.shared.usb_device;
        let serial = ctx.shared.serial;

        (usb_device, serial).lock(|usb_device, serial| {
            // USB dev poll only in the interrupt handler
            usb_device.poll(&mut [serial]);
        });
    }
}
