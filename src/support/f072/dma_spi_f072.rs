use stm32f0xx_hal::pac::{SPI1, SPI2};
use stm32f0xx_hal::spi::Spi;

pub struct DMASpi<SPI, SCKPIN, MISOPIN, MOSIPIN, WIDTH> {
    _spi: Spi<SPI, SCKPIN, MISOPIN, MOSIPIN, WIDTH>,
}

impl<SPI, SCKPIN, MISOPIN, MOSIPIN, WIDTH> DMASpi<SPI, SCKPIN, MISOPIN, MOSIPIN, WIDTH> {
    pub fn new(spi: Spi<SPI, SCKPIN, MISOPIN, MOSIPIN, WIDTH>) -> Self {
        Self { _spi: spi }
    }
}

macro_rules! dma_spi {
    ($spi: ty) => {
        impl<SCKPIN, MISOPIN, MOSIPIN, WIDTH> super::super::IDMASpi
            for DMASpi<$spi, SCKPIN, MISOPIN, MOSIPIN, WIDTH>
        {
            fn enable_dma_event(&mut self) {
                // https://github.com/stm32-rs/stm32f1xx-hal/blob/master/src/spi.rs#L747
                // spi<n>.cr2.txdmaen = true
                let spi_regs = unsafe { &*<$spi>::ptr() };
                spi_regs.cr2.modify(|_, w| w.txdmaen().set_bit());
            }

            fn dma_target_addr(&self) -> u32 {
                let spi_regs = unsafe { &*<$spi>::ptr() };
                spi_regs.dr.as_ptr() as u32
            }
        }
    };
}

dma_spi!(SPI1);
dma_spi!(SPI2);
