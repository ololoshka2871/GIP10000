use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;

use super::{
    anodes_driver::AnodesDriver, catodes_selector::CatodesSelector,
    static_buf_reader::StaticBufReader, Bus,
};

pub trait BackBufWriter {
    const COMMIT_MAGICK: u16 = u16::MAX;

    fn get_commit_magick(&self) -> u16;
    fn write(&mut self, offset: usize, data: &[u8]);
    fn commit(&mut self);
}

const ROWS_BYTES: usize = 13; // 100 // 8 + 1
pub const COLUMNS_COUNT: usize = 100;

static mut FRONT_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];
static mut BACK_BUFFER: [u8; ROWS_BYTES * COLUMNS_COUNT] = [0u8; ROWS_BYTES * COLUMNS_COUNT];

pub struct Gip10000llDriver<SPIDMA, DMA, LATCH, CB>
where
    CB: Bus<u16>,
    SPIDMA: crate::support::IDMASpi,
{
    catodes: CatodesSelector<u16, CB, COLUMNS_COUNT>,
    anodes: AnodesDriver<SPIDMA, DMA, LATCH>,

    front_buffer: &'static mut [u8],
    back_buffer: &'static mut [u8],

    col_counter: u16,
}

impl<SPIDMA, ALATCH, DMA, CB> Gip10000llDriver<SPIDMA, DMA, ALATCH, CB>
where
    SPIDMA: crate::support::IDMASpi,
    ALATCH: OutputPin<Error = Infallible>,
    DMA: crate::support::ISPITxDmaChannel,
    CB: Bus<u16>,
{
    pub fn new(
        spi: SPIDMA,
        a_latch: ALATCH,
        dma: DMA,
        catodes_bus: CB,
        pin_offsets: super::catodes_selector::Offsets<u16>,
    ) -> Self {
        unsafe {
            FRONT_BUFFER
                .iter_mut()
                .enumerate()
                .for_each(|(_i, p)| *p = 0x55);
        }

        Self {
            catodes: CatodesSelector::new(catodes_bus, pin_offsets),
            anodes: AnodesDriver::new(spi, dma, a_latch),

            front_buffer: unsafe { &mut FRONT_BUFFER },
            back_buffer: unsafe { &mut BACK_BUFFER },

            col_counter: 0,
        }
    }

    pub fn swap_buffers(&mut self) {
        cortex_m::interrupt::free(|_| {
            core::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        });
    }

    pub fn next_column(&mut self) {
        let col = self.catodes.select_column(self.col_counter);

        let from = col as usize * ROWS_BYTES;
        let to = (col as usize + 1) * ROWS_BYTES;
        let data = StaticBufReader::from(self.front_buffer[from..to].as_ptr_range());
        self.anodes.set_colum_pixels(data);
    }

    pub fn column_data_writen(&mut self) {
        self.anodes.on_spi_done();
    }

    pub fn apply_new_column(&mut self) {
        self.catodes.disable();

        let catodes = &self.catodes;
        let col_counter = self.col_counter;
        let col = self
            .anodes
            .latch_with(move || catodes.select_column(col_counter));

        self.catodes.apply_column(col);

        self.col_counter = (self.col_counter + 1) % COLUMNS_COUNT as u16;
    }
}

impl<SPIDMA, ALATCH, DMA, CB> BackBufWriter for Gip10000llDriver<SPIDMA, DMA, ALATCH, CB>
where
    SPIDMA: crate::support::IDMASpi,
    ALATCH: OutputPin<Error = Infallible>,
    DMA: crate::support::ISPITxDmaChannel,
    CB: Bus<u16>,
{
    fn write(&mut self, offset: usize, data: &[u8]) {
        if offset + data.len() <= self.back_buffer.len() {
            self.back_buffer[offset..offset + data.len()].copy_from_slice(&data);
        } else {
            // ignore request
        }
    }

    fn commit(&mut self) {
        self.swap_buffers()
    }

    fn get_commit_magick(&self) -> u16 {
        Self::COMMIT_MAGICK
    }
}
