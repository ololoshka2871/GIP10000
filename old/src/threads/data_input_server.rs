use alloc::{sync::Arc, vec::Vec};
use core::{cell::RefCell, ops::DerefMut};

use freertos_rust::{CurrentTask, Duration, Mutex};

use usb_device::UsbError;
use usbd_serial::CdcAcmClass;

use crate::output::BackBufWriter;

pub fn gcode_server<B: usb_device::bus::UsbBus, T: BackBufWriter>(
    serial_container: Arc<Mutex<&'static mut CdcAcmClass<B>>>,
    disp: &'static cortex_m::interrupt::Mutex<RefCell<Option<T>>>,
) -> ! {
    let mut data = Vec::with_capacity(64);

    loop {
        unsafe {
            data.set_len(64);
        }
        unsafe {
            let _ = freertos_rust::Task::current()
                .unwrap_unchecked()
                // ожидаем, что нотификационное значение будет > 0
                .wait_for_notification(u32::MAX, u32::MAX, Duration::infinite());
        }

        let res = if let Ok(mut p) = serial_container.lock(Duration::infinite()) {
            p.read_packet(&mut data)
        } else {
            unreachable!()
        };

        match res {
            Ok(size) => {
                unsafe {
                    data.set_len(size);
                }
                //write_responce(&serial_container, &data);

                if size > core::mem::size_of::<u16>() {
                    let mut d = [0u8; core::mem::size_of::<u16>()];
                    d.copy_from_slice(&data[..core::mem::size_of::<u16>()]);
                    let offset = u16::from_le_bytes(d);

                    cortex_m::interrupt::free(|cs| {
                        if let Some(ref mut disp) = disp.borrow(cs).borrow_mut().deref_mut() {
                            if offset == T::COMMIT_MAGICK {
                                disp.commit()
                            } else {
                                disp.write(offset as usize, &data[core::mem::size_of::<u16>()..]);
                            }
                        }
                    });
                }
            }
            Err(UsbError::WouldBlock) => {}
            Err(e) => panic!("{:?}", e),
        }
    }
}

#[allow(unused)]
pub fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: &Arc<Mutex<&'static mut CdcAcmClass<B>>>,
    data: &[u8],
) {
    loop {
        if let Ok(mut p) = serial_container.lock(Duration::zero()) {
            match p.write_packet(data) {
                Ok(_) => return,
                Err(UsbError::WouldBlock) => {}
                Err(e) => panic!("{:?}", e),
            }
        }
        CurrentTask::delay(Duration::ticks(1));
    }
}
