pub mod interrupt_controller;
pub use interrupt_controller::IInterruptController;

mod dma_cfg;
pub use dma_cfg::{DMADir, DMAWordSize};

#[cfg(feature = "stm32f072")]
mod interrupt_controller_f072;
#[cfg(feature = "stm32f072")]
pub use interrupt_controller_f072::InterruptController;

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
