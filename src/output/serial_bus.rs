use stm32f4xx_hal::{dma::StreamX, spi::Spi};

pub struct SerialBus<SPIDEV, SPIPINS, DMA, const S: u8> {
    spi: Spi<SPIDEV, SPIPINS>,
    dma_ch: StreamX<DMA, S>,
}

impl<SPIDEV, SPIPINS, DMA, const S: u8> SerialBus<SPIDEV, SPIPINS, DMA, S> {
    /*
    let mut transfer = Transfer::init_memory_to_peripheral(
        spi1_dma,
        spi1.use_dma().tx(),
        buffer,
        None,
        config::DmaConfig::default()
            .memory_increment(true)
            .fifo_enable(true)
            .fifo_error_interrupt(true)
            .transfer_complete_interrupt(true),
    );
    */

    pub fn new(spi: Spi<SPIDEV, SPIPINS>, dma_ch: StreamX<DMA, S>) -> Self {
        Self { spi, dma_ch }
    }
}

impl<SPIDEV, SPIPINS, DMA, const S: u8> super::bus::Bus<[u8]>
    for SerialBus<SPIDEV, SPIPINS, DMA, S>
{
    fn write(&mut self, data: &[u8]) {}
}
