use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;

use super::{bus::Bus, static_buf_reader::StaticBufReader};

pub struct AnodesDriver<B, LATCH> {
    bus: B,
    latch: LATCH,
}

impl<B: Bus<StaticBufReader>, LATCH: OutputPin<Error = Infallible>> AnodesDriver<B, LATCH> {
    pub fn new(anodes_bus: B, latch: LATCH) -> Self {
        Self {
            bus: anodes_bus,
            latch,
        }
    }

    pub fn set_colum_pixels(&mut self, pixels: StaticBufReader) {
        self.bus.write(pixels)
    }

    pub fn latch_with<T, F: FnOnce() -> T>(&mut self, f: F) -> T {
        use core::{sync::atomic::compiler_fence, sync::atomic::Ordering};
        let _ = self.latch.set_high();
        compiler_fence(Ordering::SeqCst);
        let res = f();
        compiler_fence(Ordering::SeqCst);
        let _ = self.latch.set_low();
        res
    }
}
