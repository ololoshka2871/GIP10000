use crate::support::interrupt_controller::{IInterruptController, Interrupt};

pub trait OnCycleFinished: Sync {
    fn cycle_finished(&self, captured: u32, target: u32, irq: Interrupt);
}

pub trait InCounter<DMA, PIN> {
    /// init timer
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut DMA,
        input: PIN, // сам пин не используется, но нужен для выведения типа и поглащается
        ic: &dyn IInterruptController,
        dma_complead: CB,
    );

    fn target() -> u32;
}

#[cfg(feature = "stm32l433")]
mod in_counters_l433;
