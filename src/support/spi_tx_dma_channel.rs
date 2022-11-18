use stm32f0xx_hal_dma::dma::Event;

use super::{DMADir, DMAWordSize};

pub trait ISPITxDmaChannel {
    /// Associated peripheral `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    fn set_peripheral_address(&mut self, address: u32, inc: bool);

    /// `address` where from/to data will be read/write
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    fn set_memory_address(&mut self, address: u32, inc: bool);

    /// Number of bytes to transfer
    fn set_transfer_length(&mut self, len: usize);

    /// Starts the DMA transfer
    fn start(&mut self);

    /// Stops the DMA transfer
    fn stop(&mut self);

    /// Returns `true` if there's a transfer in progress
    fn in_progress(&self) -> bool;

    /// Listen for dma event
    fn listen(&mut self, event: Event);

    /// Unlisten for dma event
    fn unlisten(&mut self, event: Event);

    /// Accept interrupt
    fn clear_interrupt(&mut self);

    /// Set Memory-word size
    fn set_msize(&mut self, size: DMAWordSize);

    /// Set Perith-word size
    fn set_psize(&mut self, size: DMAWordSize);

    /// Set transfer direction
    fn set_dir(&mut self, dir: DMADir);
}
