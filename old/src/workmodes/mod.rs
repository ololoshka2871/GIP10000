use freertos_rust::FreeRtosError;

pub mod high_performance_mode;

pub(crate) mod common;

pub trait WorkMode<T> {
    fn new(p: cortex_m::Peripherals, dp: stm32f4xx_hal::pac::Peripherals) -> T;
    fn start_threads(self) -> Result<(), FreeRtosError>;
    fn configure_clock(&mut self);
    fn print_clock_config(&self);
}
