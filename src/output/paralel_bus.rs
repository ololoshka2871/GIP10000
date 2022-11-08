/*
use core::marker::PhantomData;

pub struct ParalelBus<T> {
    _t: PhantomData<T>,
}

impl<T, PORT, PINS> ParalelBus<T>
where
    PORT:
{
    pub fn new(port: PORT, pins: PINS) -> Self {


        Self { _t: PhantomData }
    }
}

impl<T> super::bus::Bus<T> for ParalelBus<T> {
    fn write(&mut self, data: T) {
        todo!()
    }
}
*/

#[macro_export]
macro_rules! parralel_port {
    ($name: ident, $port: ty, $parts: ty, $regs: ty, $valuetype:ty => ($($pin:ident),+)) => {
        struct $name{
            regs_ptr: usize,
            mask: $valuetype,
        }

        impl $name {
            pub fn init(port: $port) -> Self {
                use stm32f4xx_hal::gpio::PinExt;

                let parts = port.split();
                let mut mask = 0;
                $(
                    let pin = parts.$pin.into_push_pull_output_in_state(stm32f4xx_hal::gpio::PinState::Low);
                    mask |= 1 << pin.pin_id();
                )*

                Self {
                    regs_ptr: <$port>::ptr() as _,
                    mask,
                }
            }

            pub const fn get_mask_for_pin(pin: $valuetype) -> $valuetype {
                1 << pin
            }
        }

        impl crate::output::Bus<$valuetype> for $name {
            fn write(&mut self, data: $valuetype) {
                unsafe {
                    let rb = &*(self.regs_ptr as *mut $regs);
                    rb.bsrr.write(|w| w.bits(
                        (((self.mask & !data) as u32) << 16) // reset
                        | (data as u32) //set
                    ));
                };
            }
        }

        unsafe impl Sync for $name {}
    };
}
