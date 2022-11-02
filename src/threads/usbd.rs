use core::ops::DerefMut;

use alloc::sync::Arc;
use alloc::vec::Vec;

use freertos_rust::{
    Duration, FreeRtosError, InterruptContext, Mutex, Task, TaskNotification, TaskPriority,
};
use stm32f4xx_hal::pac::interrupt;
use stm32f4xx_hal::{
    gpio::{Alternate, PushPull, PA11, PA12},
    otg_fs::{UsbBus, USB},
    pac::{self, Interrupt},
    time::Hertz,
};

use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

use crate::support::{self};

static mut EP_MEMORY: [u32; 1024] = [0; 1024];
static mut USBD_THREAD: Option<freertos_rust::Task> = None;

static mut USBD: Option<Usbd> = None;

pub struct UsbdPeriph {
    pub usb_global: pac::OTG_FS_GLOBAL,
    pub usb_device: pac::OTG_FS_DEVICE,
    pub usb_pwrclk: pac::OTG_FS_PWRCLK,
    pub pin_dm: PA11<Alternate<10, PushPull>>,
    pub pin_dp: PA12<Alternate<10, PushPull>>,
    pub hclk: Hertz,
}

pub struct Usbd {
    usb_bus: UsbBusAllocator<UsbBus<USB>>,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,

    serial: Option<SerialPort<'static, UsbBus<USB>>>,
    serial_port: Option<Arc<Mutex<&'static mut SerialPort<'static, UsbBus<USB>>>>>,
    subscribers: Vec<Task>,
}

impl Usbd {
    pub fn init(
        usbd_periph: UsbdPeriph,
        interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
        interrupt_prio: u8,
    ) {
        if unsafe { USBD.is_some() } {
            return;
        }

        defmt::info!("Creating usb low-level driver");

        let res = Self {
            usb_bus: UsbBus::new(
                USB {
                    usb_global: usbd_periph.usb_global,
                    usb_device: usbd_periph.usb_device,
                    usb_pwrclk: usbd_periph.usb_pwrclk,
                    pin_dm: usbd_periph.pin_dm,
                    pin_dp: usbd_periph.pin_dp,
                    hclk: usbd_periph.hclk,
                },
                unsafe { &mut EP_MEMORY },
            ),
            interrupt_controller,
            interrupt_prio,

            serial: None,
            serial_port: None,
            subscribers: Vec::new(),
        };

        unsafe {
            // Должен быть статик, так как заимствуется сущностью, которая будет статик.
            USBD = Some(res);
        }
    }

    fn get_static_self() -> &'static mut Usbd {
        unsafe { USBD.as_mut().expect("Call Usbd::init() first!") }
    }

    pub fn serial_port() -> Arc<Mutex<&'static mut SerialPort<'static, UsbBus<USB>>>> {
        let mut _self = Self::get_static_self();

        if _self.serial_port.is_none() {
            defmt::info!("Allocating ACM device");
            _self.serial = Some(SerialPort::new(&_self.usb_bus));

            _self.serial_port = Some(Arc::new(
                Mutex::new(_self.serial.as_mut().unwrap())
                    .expect("Failed to create serial guard mutex"),
            ));
        }
        _self.serial_port.as_ref().unwrap().clone()
    }

    pub fn subscribe(task: Task) {
        let mut _self = Self::get_static_self();

        _self.subscribers.push(task);
    }

    pub fn start(
        vid_pid: UsbVidPid,
        name: &'static str,
        manufacturer: &'static str,
        serial: &'static str,
        stack_size: usize,
        priority: TaskPriority,
    ) -> Result<(), FreeRtosError> {
        let mut _self = Self::get_static_self();

        let thread = Task::new()
            .name("Usbd")
            .stack_size((stack_size / core::mem::size_of::<u32>()) as u16)
            .priority(priority)
            .start(move |_| {
                defmt::info!("Usb thread started!");
                defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);

                let ic = &mut _self.interrupt_controller;
                {
                    defmt::trace!("Set usb interrupt prio = {}", _self.interrupt_prio);
                    ic.set_priority(Interrupt::OTG_FS.into(), _self.interrupt_prio);
                    ic.set_priority(Interrupt::OTG_FS_WKUP.into(), _self.interrupt_prio);
                }

                let serial_port = _self
                    .serial_port
                    .as_ref()
                    .expect("call Usbd::serial_port() before!");

                let mut usb_dev = UsbDeviceBuilder::new(&_self.usb_bus, vid_pid)
                    .manufacturer(manufacturer)
                    .product(name)
                    .serial_number(serial)
                    .composite_with_iads()
                    .build();

                defmt::info!("USB ready!");

                loop {
                    // Важно! Список передаваемый сюда в том же порядке,
                    // что были инициализированы интерфейсы
                    let res = match serial_port.lock(Duration::ms(1)) {
                        Ok(mut serial) => usb_dev.poll(&mut [*serial.deref_mut()]),
                        Err(_) => true,
                    };

                    if res {
                        // crate::support::led::led_set(1);
                        _self
                            .subscribers
                            .iter()
                            .for_each(|s| s.notify(TaskNotification::Increment));

                        // support::mast_yield();
                    } else {
                        // crate::support::led::led_set(0);

                        // block until usb interrupt
                        cortex_m::interrupt::free(|_| {
                            ic.unmask(Interrupt::OTG_FS.into());
                            ic.unmask(Interrupt::OTG_FS_WKUP.into());
                        });

                        unsafe {
                            let _ = freertos_rust::Task::current()
                                .unwrap_unchecked()
                                // ожидаем, что нотификационное значение будет > 0
                                .wait_for_notification(u32::MAX, u32::MAX, Duration::infinite());
                        }

                        cortex_m::interrupt::free(|_| {
                            ic.mask(Interrupt::OTG_FS.into());
                            ic.mask(Interrupt::OTG_FS_WKUP.into());
                        });
                    }
                }
            })?;

        unsafe {
            USBD_THREAD = Some(thread);
        }

        Ok(())
    }
}

// USB exception
// ucCurrentPriority >= ucMaxSysCallPriority (80)

#[interrupt]
unsafe fn OTG_FS() {
    use cortex_m::peripheral::NVIC;

    usb_interrupt();

    NVIC::mask(Interrupt::OTG_FS);
    NVIC::unpend(Interrupt::OTG_FS);
}

#[interrupt]
unsafe fn OTG_FS_WKUP() {
    use cortex_m::peripheral::NVIC;

    usb_interrupt();

    NVIC::mask(Interrupt::OTG_FS_WKUP);
    NVIC::unpend(Interrupt::OTG_FS_WKUP);
}

unsafe fn usb_interrupt() {
    let interrupt_ctx = InterruptContext::new();
    if let Some(usbd) = USBD_THREAD.as_ref() {
        // Результат не особо важен
        // инкремент нотификационного значения
        let _ = usbd.notify_from_isr(&interrupt_ctx, TaskNotification::Increment);
    }

    // Как только прерывание случилось, мы посылаем сигнал потоку
    // НО ивент вызвавший прерыывание пока не снялся, поэтому мы будем
    // бесконечно в него заходить по кругу, нужно запретить пока что это
    // прерывание
    // TODO: device independent layer
    // cortex_m::peripheral::NVIC::mask(Interrupt::USB...);
    // cortex_m::peripheral::NVIC::unpend(Interrupt::USB...);
}
