use core::convert::Infallible;

use stm32f4xx_hal::{
    dma::{
        self,
        traits::{self, StreamISR},
        ChannelX, StreamX,
    },
    spi::Spi,
    timer::{self, CounterUs},
};

use super::{
    anodes_driver::AnodesDriver, catodes_selector::CatodesSelector, paralel_bus::ParalelBus,
    serial_bus::SerialBus, static_buf_reader::StaticBufReader,
};

const ROWS_BYTES: usize = 13; // 100 // 8 + 1
const COLUMNS_COUNT: usize = 100;

static mut FRONT_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];
static mut BACK_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];

pub struct Gip10000llDriver<SPIDEV, SPIPINS, ALATCH, TIM, DMA, const S: u8>
where
    DMA: stm32f4xx_hal::dma::traits::Instance,
    StreamX<DMA, S>: StreamISR,
    ChannelX<S>: stm32f4xx_hal::dma::traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>:
        traits::DMASet<StreamX<DMA, S>, S, stm32f4xx_hal::dma::MemoryToPeripheral>,
    SPIDEV: stm32f4xx_hal::spi::Instance,
{
    catodes: CatodesSelector<u8, ParalelBus<u8>>,
    anodes: AnodesDriver<SerialBus<SPIDEV, SPIPINS, DMA, S>, ALATCH>,
    timer: CounterUs<TIM>,

    front_buffer: &'static mut [u8],
    back_buffer: &'static mut [u8],

    col_counter: u8,
}

impl<SPIDEV, SPIPINS, ALATCH, TIM, DMA, const S: u8>
    Gip10000llDriver<SPIDEV, SPIPINS, ALATCH, TIM, DMA, S>
where
    DMA: stm32f4xx_hal::dma::traits::Instance,
    StreamX<DMA, S>: StreamISR,
    ChannelX<S>: stm32f4xx_hal::dma::traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>:
        traits::DMASet<StreamX<DMA, S>, S, stm32f4xx_hal::dma::MemoryToPeripheral>,
    SPIDEV: stm32f4xx_hal::spi::Instance,
    ALATCH: embedded_hal::digital::v2::OutputPin<Error = Infallible>,
    TIM: stm32f4xx_hal::timer::Instance,
{
    pub fn new(
        timer: CounterUs<TIM>,
        spi: Spi<SPIDEV, SPIPINS>,
        a_latch: ALATCH,
        dma_ch: StreamX<DMA, S>,
    ) -> Self {
        Self {
            catodes: CatodesSelector::new(ParalelBus::new()),
            anodes: AnodesDriver::new(SerialBus::new(spi, dma_ch), a_latch),
            timer,

            front_buffer: unsafe { &mut FRONT_BUFFER },
            back_buffer: unsafe { &mut BACK_BUFFER },

            col_counter: 0,
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() <= self.back_buffer.len() {
            self.back_buffer[offset..offset + data.len()].copy_from_slice(&data);
        } else {
            // ignore request
        }
    }

    pub fn swap_buffers(&mut self) {
        let _ = freertos_rust::CriticalRegion::enter();
        core::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
    }

    pub fn start(&mut self) {
        use stm32f4xx_hal::prelude::*;
        self.timer.start(10.micros()).unwrap();
    }

    fn next_column(&mut self) {
        let col = self.catodes.select_column(self.col_counter);

        let from = col as usize * ROWS_BYTES;
        let to = col as usize * (ROWS_BYTES + 1);
        let data = StaticBufReader::from(self.front_buffer[from..to].as_ptr_range());
        self.anodes.set_colum_pixels(data);
    }

    pub fn on_timer(&mut self) {
        use stm32f4xx_hal::timer::Event;

        self.timer.clear_interrupt(Event::Update);

        self.next_column()
    }

    pub fn on_dma(&mut self) {
        self.catodes.disable();

        let catodes = &self.catodes;
        let col_counter = self.col_counter;
        let col = self
            .anodes
            .latch_with(move || catodes.select_column(col_counter));

        self.catodes.enable_with(col);

        self.col_counter = (self.col_counter + 1) % COLUMNS_COUNT as u8;
    }
}

unsafe impl<SPIDEV, SPIPINS, ALATCH, TIM, DMA, const S: u8> Sync
    for Gip10000llDriver<SPIDEV, SPIPINS, ALATCH, TIM, DMA, S>
where
    DMA: stm32f4xx_hal::dma::traits::Instance,
    StreamX<DMA, S>: StreamISR,
    ChannelX<S>: stm32f4xx_hal::dma::traits::Channel,
    stm32f4xx_hal::spi::Tx<SPIDEV>:
        traits::DMASet<StreamX<DMA, S>, S, stm32f4xx_hal::dma::MemoryToPeripheral>,
    SPIDEV: stm32f4xx_hal::spi::Instance,
{
}
