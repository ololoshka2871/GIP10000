use core::marker::PhantomData;

use stm32f4xx_hal::{
    dma::{self, traits, ChannelX, MemoryToPeripheral, StreamX, Transfer},
    spi::{Spi, Tx},
};

use super::static_buf_reader::StaticBufReader;

pub struct SerialBus<SPIDEV, SPIPINS, DMA, const S: u8>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    transfer: Transfer<StreamX<DMA, S>, S, Tx<SPIDEV>, MemoryToPeripheral, StaticBufReader>,
    _pins: PhantomData<SPIPINS>,
}

impl<SPIDEV, SPIPINS, DMA, const S: u8> SerialBus<SPIDEV, SPIPINS, DMA, S>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    pub fn new(spi: Spi<SPIDEV, SPIPINS>, dma_ch: StreamX<DMA, S>) -> Self {
        Self {
            transfer: Transfer::init_memory_to_peripheral(
                dma_ch,
                spi.use_dma().tx(),
                StaticBufReader::empty(),
                None,
                dma::config::DmaConfig::default()
                    .memory_increment(true)
                    .fifo_enable(true)
                    .fifo_error_interrupt(true)
                    .transfer_complete_interrupt(true),
            ),
            _pins: PhantomData,
        }
    }
}

impl<SPIDEV, SPIPINS, DMA, const S: u8> super::bus::Bus<StaticBufReader>
    for SerialBus<SPIDEV, SPIPINS, DMA, S>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    fn write(&mut self, data: StaticBufReader) {
        self.transfer.next_transfer(data).unwrap();
    }
}
