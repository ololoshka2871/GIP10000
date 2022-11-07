use core::marker::PhantomData;

use stm32f4xx_hal::{
    dma::{
        self,
        traits::{self, PeriAddress},
        ChannelX, MemoryToPeripheral, StreamX, Transfer,
    },
    spi::{Spi, Tx},
};

use super::static_buf_reader::StaticBufReader;

pub struct SPIBus<SPIDEV, SPIPINS, DMA, const S: u8>
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

impl<SPIDEV, SPIPINS, DMA, const S: u8> SPIBus<SPIDEV, SPIPINS, DMA, S>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    pub fn new(mut spi: Spi<SPIDEV, SPIPINS>, dma_ch: StreamX<DMA, S>) -> Self {
        //cortex_m::prelude::_embedded_hal_blocking_spi_Write::write(&mut spi, &[0]).unwrap();
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
    for SPIBus<SPIDEV, SPIPINS, DMA, S>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    fn write(&mut self, data: StaticBufReader) {
        self.transfer.next_transfer(data).unwrap();
        /* триггерить на самом деле не надо, видимо то, что SPI готов к передаче - это флаг TXE и его достаточно для DMA
        self.transfer.start(|ch| unsafe {
            let dr = &*(ch.address() as *mut stm32f4xx_hal::pac::spi1::DR);
            dr.write(|w| w.bits(0x55));
        });
        */
    }
}
