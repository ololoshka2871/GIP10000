pub mod interrupt_controller;
pub use interrupt_controller::IInterruptController;

#[cfg(feature = "stm32f072")]
mod interrupt_controller_f072;

#[cfg(feature = "stm32f072")]
pub use interrupt_controller_f072::InterruptController;
