use core::{convert::Infallible, marker::PhantomData};

use cortex_m::interrupt::InterruptNumber;
use embedded_hal::digital::v2::OutputPin;
use stm32f4xx_hal::{
    dma::{self, traits, ChannelX, MemoryToPeripheral, StreamX, Transfer},
    spi::{Spi, Tx},
};

use super::static_buf_reader::StaticBufReader;

pub struct AnodesDriver<SPIDEV, SPIPINS, DMA, LATCH, I, const S: u8>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
{
    transfer: Transfer<StreamX<DMA, S>, S, Tx<SPIDEV>, MemoryToPeripheral, StaticBufReader>,
    latch: LATCH,
    second_isr: bool,
    irq_n: I,

    _pins: PhantomData<SPIPINS>,
}

impl<SPIDEV, SPIPINS, DMA, LATCH: OutputPin<Error = Infallible>, I, const S: u8>
    AnodesDriver<SPIDEV, SPIPINS, DMA, LATCH, I, S>
where
    SPIDEV: stm32f4xx_hal::spi::Instance,
    DMA: traits::Instance,
    StreamX<DMA, S>: traits::StreamISR,
    ChannelX<S>: traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>: traits::DMASet<StreamX<DMA, S>, S, dma::MemoryToPeripheral>,
    I: InterruptNumber,
{
    pub fn new(spi: Spi<SPIDEV, SPIPINS>, dma_ch: StreamX<DMA, S>, latch: LATCH, irq_n: I) -> Self {
        Self {
            transfer: Transfer::init_memory_to_peripheral(
                dma_ch,
                spi.use_dma().tx(),
                StaticBufReader::empty(),
                None,
                dma::config::DmaConfig::default().memory_increment(true),
            ),
            latch,
            second_isr: false,
            irq_n,
            _pins: PhantomData,
        }
    }

    pub fn set_colum_pixels(&mut self, pixels: StaticBufReader) {
        self.transfer.clear_interrupts();
        self.transfer.next_transfer(pixels).unwrap();
        unsafe {
            cortex_m::peripheral::NVIC::unmask(self.irq_n);
            self.second_isr = false;
        }
        /* триггерить на самом деле не надо, видимо то, что SPI готов к передаче - это флаг TXE и его достаточно для DMA
        self.transfer.start(|ch| unsafe {
            let dr = &*(ch.address() as *mut stm32f4xx_hal::pac::spi1::DR);
            dr.write(|w| w.bits(0x55));
        });
        */
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

    pub fn on_spi_isr(&mut self) -> bool {
        if self.second_isr {
            cortex_m::peripheral::NVIC::mask(self.irq_n);
        } else {
            self.transfer.clear_interrupts();
        }
        let res = self.second_isr;
        self.second_isr = !self.second_isr;
        res
    }
}
