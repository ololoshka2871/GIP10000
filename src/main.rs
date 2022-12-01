#![no_main]
#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(slice_from_ptr_range)]

mod output;
mod support;

use panic_abort as _;
use rtic::app;

use stm32f0xx_hal::gpio::gpioa::{PA0, PA1, PA5, PA6, PA7};
use stm32f0xx_hal::gpio::{Alternate, Output, PushPull, AF0};
use stm32f0xx_hal::pac::{gpiof, GPIOB, SPI1, TIM17};
use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::spi::{self, EightBit, Spi};
use stm32f0xx_hal::usb::{Peripheral, UsbBusType};

use stm32f0xx_hal_dma::dma::{dma1::C3, DmaExt};

use systick_monotonic::Systick;

use stm32_usbd::UsbBus;
use usb_device::class_prelude::*;
use usb_device::prelude::*;

use usbd_serial::CdcAcmClass;

use support::{DMASpi, SPITxDmaChannel};

const CDC_POCKET_SIZE: u16 = 64;

const SPI_CLOCK_KHZ: u32 = 1000;
const FPS: u32 = 30;
const COLUMN_TICK_RATE_HZ: u32 = crate::output::COLUMNS_COUNT as u32 * FPS;
const POST_DMA_DELAY_HZ: u32 = 7500; // Подбирается экспериментально, чтобы latch был после SPI посылки

parralel_port!(Catodes, GPIOB, gpiob::Parts, gpiof::RegisterBlock,
    u16 => ([pb3: 3], [pb4: 4], [pb5: 5], [pb6: 6], [pb7: 7], [pb8: 8], [pb12: 12], [pb13: 13])
);

#[app(device = stm32f0xx_hal::pac, peripherals = true, dispatchers = [ADC_COMP])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        gip10k: crate::output::Gip10000llDriver<
            DMASpi<SPI1, PA5<Alternate<AF0>>, PA6<Alternate<AF0>>, PA7<Alternate<AF0>>, EightBit>,
            SPITxDmaChannel<C3>,
            PA1<Output<PushPull>>,
            Catodes,
        >,
    }

    #[local]
    struct Local {
        usb_device: UsbDevice<'static, UsbBusType>,
        serial: CdcAcmClass<'static, UsbBus<Peripheral>>,
        col_timer: stm32f0xx_hal::timers::Timer<TIM17>,

        led: PA0<Output<PushPull>>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<100>;

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

        //---------------------------------------------------------------------

        let gpioa = cx.device.GPIOA.split(&mut rcc);

        let (sck, miso, mosi, pin_dm, pin_dp, mut latch, led) = cortex_m::interrupt::free(|cs| {
            (
                gpioa.pa5.into_alternate_af0(cs),
                gpioa.pa6.into_alternate_af0(cs),
                gpioa.pa7.into_alternate_af0(cs),
                gpioa.pa11,
                gpioa.pa12,
                gpioa.pa1.into_push_pull_output_hs(cs),
                gpioa.pa0.into_push_pull_output(cs),
            )
        });

        // usb
        let usb = stm32f0xx_hal::usb::Peripheral {
            usb: cx.device.USB,
            pin_dm,
            pin_dp,
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

        //---------------------------------------------------------------------

        let mut spi1 = Spi::spi1(
            cx.device.SPI1,
            (sck, miso, mosi),
            spi::Mode {
                polarity: spi::Polarity::IdleLow,
                phase: spi::Phase::CaptureOnFirstTransition,
            },
            SPI_CLOCK_KHZ.khz(),
            &mut rcc,
        );

        {
            // force reset anodes
            let _ = spi1.write(&[0u8; 16]);
            let _ = latch.set_low();
            for _ in 1..100 {
                cortex_m::asm::nop();
            }
            let _ = latch.set_high();
        }

        let mut spi1_dma = cx.device.DMA1.split(&mut rcc).3; // SPI1_TX
        spi1_dma.listen(stm32f0xx_hal_dma::dma::Event::TransferComplete);

        let mut col_timer = stm32f0xx_hal::timers::Timer::tim17(
            cx.device.TIM17,
            COLUMN_TICK_RATE_HZ.hz(),
            &mut rcc,
        );

        col_timer.listen(stm32f0xx_hal::timers::Event::TimeOut);

        let gip10k = output::Gip10000llDriver::new(
            support::DMASpi::new(spi1),
            latch,
            support::SPITxDmaChannel::new(spi1_dma),
            Catodes::init(cx.device.GPIOB, &mut rcc, true),
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

        (
            Shared { gip10k },
            Local {
                usb_device: usb_dev,
                serial,
                col_timer,
                led,
            },
            init::Monotonics(mono),
        )
    }

    //-------------------------------------------------------------------------

    #[task(binds = USB, shared = [gip10k], local = [usb_device, serial, led], priority = 1)]
    fn usb_handler(mut ctx: usb_handler::Context) {
        let _ = ctx.local.led.set_high();
        let usb_device = ctx.local.usb_device;
        let serial = ctx.local.serial;

        // USB dev poll only in the interrupt handler
        if usb_device.poll(&mut [serial]) {
            let mut data = [0u8; CDC_POCKET_SIZE as usize];

            match serial.read_packet(&mut data) {
                Ok(size) if size > core::mem::size_of::<u16>() => {
                    let mut d = [0u8; core::mem::size_of::<u16>()];
                    d.copy_from_slice(&data[..core::mem::size_of::<u16>()]);
                    let offset = u16::from_le_bytes(d);

                    ctx.shared.gip10k.lock(|gip10k| {
                        use crate::output::BackBufWriter;

                        if offset == gip10k.get_commit_magick() {
                            gip10k.commit()
                        } else {
                            gip10k.write(offset as usize, &data[core::mem::size_of::<u16>()..]);
                        }
                    });
                }

                Ok(size) if size > 0 => {
                    let _ = serial.write_packet(&data);
                }

                _ => {}
            }
        }
        let _ = ctx.local.led.set_low();
    }

    // next column
    #[task(binds = TIM17, shared = [gip10k], local = [col_timer, t: bool = false], priority = 3)]
    fn tim17_handler(mut ctx: tim17_handler::Context) {
        let _ = ctx.local.col_timer.wait(); // clear it flag
        if *ctx.local.t {
            ctx.shared.gip10k.lock(|gip10k| gip10k.apply_new_column());
            ctx.local.col_timer.start(COLUMN_TICK_RATE_HZ.hz());
        } else {
            ctx.shared.gip10k.lock(|gip10k| gip10k.next_column());
            ctx.local.col_timer.start(POST_DMA_DELAY_HZ.hz());
        }
        *ctx.local.t = !*ctx.local.t;
    }

    // column writen
    #[task(binds = DMA1_CH2_3, shared = [gip10k], priority = 2)]
    fn dma1_ch2_3_handler(mut ctx: dma1_ch2_3_handler::Context) {
        ctx.shared.gip10k.lock(|gip10k| gip10k.column_data_writen());
    }

    //-------------------------------------------------------------------------

    #[idle(/*local = [led]*/)]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            //let _ = ctx.local.led.set_high();
            cortex_m::asm::wfi();
            //let _ = ctx.local.led.set_low();
        }
    }
}
