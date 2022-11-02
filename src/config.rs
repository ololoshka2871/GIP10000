//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 25_000_000;

//-----------------------------------------------------------------------------
// Это же число должно быть записано в src/configTemplate/FreeRTOSConfig.h через build.rs

pub const FREERTOS_CONFIG_FREQ: u32 = 60_000_000; // /1

//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 10;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = MASTER_COUNTER_INTERRUPT_PRIO + 1;

// dma value captured interrupt prio
pub const DMA_IRQ_PRIO: u8 = IRQ_HIGEST_PRIO + 5;

//-----------------------------------------------------------------------------

// Приоритеты, обльше -> лучше

/// pseudo-idle task prio
pub const IDLE_TASK_PRIO: u8 = 0;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = IDLE_TASK_PRIO + 3;

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = IDLE_TASK_PRIO + 1;

//-----------------------------------------------------------------------------

/// monitor stack size
pub const MONITOR_TASK_STACK_SIZE: usize = 2048 + 2048;
