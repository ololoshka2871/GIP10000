use core::convert::Infallible;

use embedded_dma::ReadBuffer;
use embedded_hal::digital::v2::OutputPin;

use crate::support::{DMADir, DMAWordSize};

use super::static_buf_reader::StaticBufReader;

pub struct AnodesDriver<SPIDMA, DMA, LATCH>
where
    SPIDMA: crate::support::IDMASpi,
{
    _spi: SPIDMA,
    dma: DMA,
    latch: LATCH,
}

impl<SPIDMA, DMA, LATCH> AnodesDriver<SPIDMA, DMA, LATCH>
where
    SPIDMA: crate::support::IDMASpi,
    LATCH: OutputPin<Error = Infallible>,
    DMA: crate::support::ISPITxDmaChannel,
{
    pub fn new(mut spi: SPIDMA, mut dma: DMA, latch: LATCH) -> Self {
        dma.stop();
        dma.set_dir(DMADir::Mem2Perith);
        dma.set_peripheral_address(spi.dma_target_addr(), false);
        dma.set_msize(DMAWordSize::S8Bit);
        dma.set_psize(DMAWordSize::S8Bit);

        spi.enable_dma_event();

        Self {
            _spi: spi,
            dma,
            latch,
        }
    }

    pub fn set_colum_pixels(&mut self, pixels: StaticBufReader) {
        self.dma.stop();
        self.dma.clear_interrupt();

        let (ptr, len) = unsafe { pixels.read_buffer() };
        self.dma.set_memory_address(ptr as u32, true);
        self.dma.set_transfer_length(len);
        self.dma.start();
    }

    pub fn latch_with<T, F: FnOnce() -> T>(&mut self, f: F) -> T {
        use core::{sync::atomic::compiler_fence, sync::atomic::Ordering};

        let _ = self.latch.set_low();
        compiler_fence(Ordering::SeqCst);
        let res = f();
        compiler_fence(Ordering::SeqCst);
        let _ = self.latch.set_high();
        res
    }

    pub fn on_spi_done(&mut self) {
        self.dma.clear_interrupt();
    }
}
