//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 1;

// dma value captured interrupt prio
pub const DMA_IRQ_PRIO: u8 = IRQ_HIGEST_PRIO + 5;

/// column update counter interrupt prio
pub const UPDATE_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 6;

//-----------------------------------------------------------------------------
