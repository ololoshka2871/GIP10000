use alloc::{sync::Arc, vec::Vec};

use freertos_rust::{CurrentTask, Duration, Mutex};

use usb_device::UsbError;
use usbd_serial::CdcAcmClass;

pub fn gcode_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<&'static mut CdcAcmClass<B>>>,
    // gcode_tx_queue: Arc<Queue<GCode>>,
    // req_tx_queue: Arc<Queue<Request>>,
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
                write_responce(&serial_container, &data);
            }
            Err(UsbError::WouldBlock) => {}
            Err(e) => panic!("{:?}", e),
        }
    }
}

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
