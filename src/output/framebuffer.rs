use super::{
    anodes_driver::AnodesDriver, catodes_selector::CatodesSelector, paralel_bus::ParalelBus,
    serial_bus::SerialBus,
};

const ROWS_BYTES: usize = 13; // 100 // 8 + 1
const COLUMNS_COUNT: usize = 100;

static mut FRONT_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];
static mut BACK_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];

pub struct Framebuffer {
    catodes: CatodesSelector<u8, ParalelBus<u8>>,
    anodes: AnodesDriver<SerialBus>,
    //timer: Timer,
    front_buffer: &'static mut [u8],
    back_buffer: &'static mut [u8],
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            catodes: CatodesSelector::new(ParalelBus::new()),
            anodes: AnodesDriver::new(SerialBus::new()),
            front_buffer: unsafe { &mut FRONT_BUFFER },
            back_buffer: unsafe { &mut BACK_BUFFER },
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() <=  self.back_buffer.len() {
            self.back_buffer[offset..offset+data.len()].copy_from_slice(&data);
        } else {
            // ignore request
        }
    }

    pub fn swap_buffers(&mut self) {
        let _ = freertos_rust::CriticalRegion::enter();
        core::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
    }

    pub fn start(&mut self) {
        todo!()
    }
}
