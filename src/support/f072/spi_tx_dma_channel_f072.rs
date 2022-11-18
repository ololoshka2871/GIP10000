use stm32f0xx_hal_dma::dma::{
    dma1::{C3, C5},
    Event,
};

use crate::support::{DMADir, DMAWordSize};

pub struct SPITxDmaChannel<DMACH>(DMACH);

impl<DMACH> SPITxDmaChannel<DMACH> {
    pub fn new(ch: DMACH) -> Self {
        Self(ch)
    }
}

macro_rules! make_spi_tx_channel {
    ($ch: ty, $cgifn: ident) => {
        impl super::super::ISPITxDmaChannel for SPITxDmaChannel<$ch> {
            fn set_peripheral_address(&mut self, address: u32, inc: bool) {
                self.0.set_peripheral_address(address, inc);
            }

            fn set_memory_address(&mut self, address: u32, inc: bool) {
                self.0.set_memory_address(address, inc);
            }

            fn set_transfer_length(&mut self, len: usize) {
                self.0.set_transfer_length(len);
            }

            fn start(&mut self) {
                self.0.start();
            }

            fn stop(&mut self) {
                self.0.stop();
            }

            fn in_progress(&self) -> bool {
                self.0.in_progress()
            }

            fn listen(&mut self, event: Event) {
                self.0.listen(event)
            }

            fn unlisten(&mut self, event: Event) {
                self.0.unlisten(event)
            }

            fn clear_interrupt(&mut self) {
                self.0.ifcr().write(|w| w.$cgifn().set_bit())
            }

            fn set_msize(&mut self, size: DMAWordSize) {
                self.0
                    .ch()
                    .cr
                    .modify(|_, w| unsafe { w.msize().bits(size as u8) })
            }

            fn set_psize(&mut self, size: DMAWordSize) {
                self.0
                    .ch()
                    .cr
                    .modify(|_, w| unsafe { w.psize().bits(size as u8) })
            }

            fn set_dir(&mut self, dir: DMADir) {
                self.0
                    .ch()
                    .cr
                    .modify(|_, w| w.dir().bit(dir == DMADir::Mem2Perith));
            }
        }
    };
}

make_spi_tx_channel!(C3, cgif3);
make_spi_tx_channel!(C5, cgif5);
