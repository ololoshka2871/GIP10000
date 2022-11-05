use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;

use super::bus::Bus;

pub struct AnodesDriver<B, LATCH> {
    bus: B,
    latch: LATCH,
}

impl<B: Bus<[u8]>, LATCH: OutputPin<Error = Infallible>> AnodesDriver<B, LATCH> {
    pub fn new(anodes_bus: B, latch: LATCH) -> Self {
        Self {
            bus: anodes_bus,
            latch,
        }
    }

    pub fn set_colum_pixels(&mut self, pixels: &[u8]) {
        todo!()
    }

    pub fn set_colum_pixels_and_then<F: FnOnce()>(&mut self, pixels: &[u8], f: F) {
        todo!()
    }
}
