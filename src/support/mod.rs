mod dma_cfg;
pub use dma_cfg::{DMADir, DMAWordSize};

mod dma_spi;
pub use dma_spi::IDMASpi;

#[cfg(feature = "stm32f072")]
mod dma_spi_f072;
#[cfg(feature = "stm32f072")]
pub use dma_spi_f072::DMASpi;

mod spi_tx_dma_channel;
pub use spi_tx_dma_channel::ISPITxDmaChannel;

#[cfg(feature = "stm32f072")]
mod spi_tx_dma_channel_f072;
#[cfg(feature = "stm32f072")]
pub use spi_tx_dma_channel_f072::SPITxDmaChannel;
