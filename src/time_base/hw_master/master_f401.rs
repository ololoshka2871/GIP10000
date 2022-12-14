use stm32f4xx_hal::pac::interrupt;
use stm32f4xx_hal::pac::{tim10, Interrupt as IRQ, RCC};

use crate::support::interrupt_controller::{IInterruptController, Interrupt};

use super::MasterCounterInfo;

struct GeneralPurpoceCounter10 {
    id: u8,
}

impl GeneralPurpoceCounter10 {
    fn tim(&self) -> &'static tim10::RegisterBlock {
        unsafe { &*(0x4001_4400 as *const tim10::RegisterBlock) } // stm32f1-0.14.0/src/stm32f103/mod.rs:942
    }

    fn interrupt_n(&self) -> Interrupt {
        IRQ::TIM1_UP_TIM10.into()
    }
}

impl MasterCounterInfo for GeneralPurpoceCounter10 {
    fn id(&self) -> u32 {
        self.id as u32
    }

    // stm32l4xx-hal-0.6.0/src/timer.rs
    fn init(&self) {
        let enr = unsafe { &(*RCC::ptr()).apb2enr };
        let rstr = unsafe { &(*RCC::ptr()).apb2rstr };

        // tim10 - apb2
        enr.modify(|_, w| w.tim10en().set_bit());
        rstr.modify(|_, w| w.tim10rst().set_bit());
        rstr.modify(|_, w| w.tim10rst().clear_bit());
    }

    fn set_interrupt_prio(&self, controller: &dyn IInterruptController, prio: u8) {
        controller.set_priority(self.interrupt_n(), prio);
    }

    fn start(&self) {
        // pause
        self.stop();

        let tim = self.tim();

        // no prescaler
        tim.psc.write(|w| unsafe { w.bits(0) });

        // autoreload
        tim.arr.write(|w| unsafe { w.bits(u16::MAX as u32) });

        // Trigger an update event to load the prescaler value to the clock
        tim.egr.write(|w| w.ug().set_bit());

        // enable UIF_CPY
        tim.cr1
            .modify(|r, w| unsafe { w.bits(r.bits() | 1u32 << 11) });

        // start counter
        tim.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&self) {
        self.tim().cr1.modify(|_, w| w.cen().clear_bit());
    }

    fn clear_interrupt(&self, controller: &dyn IInterruptController) {
        self.tim().sr.write(|w| w.uif().clear_bit());
        controller.unpend(self.interrupt_n());
    }

    fn enable_interrupt(&self, controller: &dyn IInterruptController, enable: bool) {
        let irq = self.interrupt_n();
        if enable {
            controller.unmask(irq);
        } else {
            controller.mask(irq);
        }

        self.tim().dier.write(|w| w.uie().bit(enable));
    }

    fn value(&self) -> u32 {
        self.tim().cnt.read().bits() & (u16::MAX as u32)
    }

    fn cnt_addr(&self) -> usize {
        &self.tim().cnt as *const _ as usize
    }

    fn uif_cpy_mask(&self) -> Option<u32> {
        None // not supported in f401
    }

    fn is_irq_pending(&self, controller: &dyn IInterruptController) -> bool {
        controller.is_pending(self.interrupt_n())
    }
}

pub(crate) static MASTER_LIST: [&dyn MasterCounterInfo; 1] = [&GeneralPurpoceCounter10 { id: 10 }];

#[interrupt]
unsafe fn TIM1_UP_TIM10() {
    crate::time_base::master_counter::master_ovf(10);
}
