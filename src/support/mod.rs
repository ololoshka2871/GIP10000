mod dma_cfg;
pub use dma_cfg::{DMADir, DMAWordSize};

mod dma_spi;
pub use dma_spi::IDMASpi;

mod spi_tx_dma_channel;
pub use spi_tx_dma_channel::ISPITxDmaChannel;

#[cfg(feature = "stm32f072")]
mod f072;
#[cfg(feature = "stm32f072")]
pub use f072::{DMASpi, SPITxDmaChannel};
