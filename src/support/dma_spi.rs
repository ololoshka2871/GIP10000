pub trait IDMASpi {
    fn enable_dma_event(&mut self);
    fn dma_target_addr(&self) -> u32;
}
