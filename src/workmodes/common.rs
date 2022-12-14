use freertos_rust::Duration;
use stm32f4xx_hal::time::Hertz;

pub fn print_clock_config(clocks: &stm32f4xx_hal::rcc::Clocks) {
    defmt::info!(
        "Clock config: CPU={} Mhz, pclk1={} MHz, pclk2={}MHz",
        clocks.hclk().to_MHz(),
        clocks.pclk1().to_MHz(),
        clocks.pclk2().to_MHz(),
    );
}

pub fn create_monitor(_sysclk: Hertz) -> Result<(), freertos_rust::FreeRtosError> {
    #[cfg(feature = "monitor")]
    #[cfg(debug_assertions)]
    {
        use crate::threads;
        use freertos_rust::{Task, TaskPriority};

        pub static MONITOR_MSG_PERIOD: u32 = 1000;

        defmt::trace!("Creating monitor thread...");
        let monitoring_period = Duration::ms(MONITOR_MSG_PERIOD);
        Task::new()
            .name("Monitord")
            .stack_size(
                (crate::config::MONITOR_TASK_STACK_SIZE / core::mem::size_of::<u32>()) as u16,
            )
            .priority(TaskPriority(crate::config::MONITOR_TASK_PRIO))
            .start(move |_| threads::monitor::monitord(monitoring_period))?;
    }

    Ok(())
}
