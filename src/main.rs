#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

extern crate alloc;

//mod main_data_storage;
//mod protobuf;
//mod sensors;
//mod settings;
mod support;
mod threads;
mod time_base;
mod workmodes;

pub mod config;
//pub mod config_pins;

#[cfg(debug_assertions)]
mod master_value_stat;

use cortex_m_rt::entry;

use stm32f4xx_hal::pac;

use panic_abort as _;

use crate::{
    support::free_rtos_error_ext::FreeRtosErrorContainer,
    workmodes::high_performance_mode::HighPerformanceMode,
};
use workmodes::WorkMode;

//use crate::support::free_rtos_error_ext::FreeRtosErrorContainer;

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: freertos_rust::FreeRtosAllocator = freertos_rust::FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    // #[cfg(debug_assertions)]
    // cortex_m::asm::bkpt();

    defmt::trace!("++ Start up! ++");

    let p = unsafe { cortex_m::Peripherals::take().unwrap_unchecked() };
    let dp = unsafe { pac::Peripherals::take().unwrap_unchecked() };

    start_at_mode::<HighPerformanceMode>(p, dp)
        .unwrap_or_else(|e| defmt::panic!("Failed to start thread: {}", FreeRtosErrorContainer(e)));

    freertos_rust::FreeRtosUtils::start_scheduler();
}

fn start_at_mode<T>(
    p: cortex_m::Peripherals,
    dp: pac::Peripherals,
) -> Result<(), freertos_rust::FreeRtosError>
where
    T: WorkMode<T>,
{
    let mut mode = T::new(p, dp);
    mode.configure_clock();
    mode.print_clock_config();

    #[cfg(debug_assertions)]
    master_value_stat::init_master_getter(time_base::master_counter::MasterCounter::acquire());

    mode.start_threads()
}

//-----------------------------------------------------------------------------
