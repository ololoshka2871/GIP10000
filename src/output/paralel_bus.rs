#[macro_export]
macro_rules! parralel_port {
    ($name: ident, $port: ty, $parts: ty, $regs: ty, $valuetype:ty => ($([$pin:ident: $bit_n:expr]),+)) => {
        pub struct $name{
            regs_ptr: usize,
            mask: $valuetype,
            invert: bool,
        }

        impl $name {
            pub fn init(port: $port, rcc: &mut stm32f0xx_hal::rcc::Rcc, invert: bool) -> Self {
                let parts = port.split(rcc);
                let mut mask = 0;

                cortex_m::interrupt::free(|cs| {
                    $(
                        let mut pin = parts.$pin.into_push_pull_output(cs);
                        if invert {
                            let _ = pin.set_high();
                        }
                        mask |= 1 << ($bit_n);
                    )*
               });

                Self {
                    regs_ptr: <$port>::ptr() as _,
                    mask,
                    invert,
                }
            }

            pub const fn get_mask_for_pin(pin: $valuetype) -> $valuetype {
                1 << pin
            }
        }

        impl crate::output::Bus<$valuetype> for $name {
            fn write(&mut self, mut data: $valuetype) {
                unsafe {
                    let rb = &*(self.regs_ptr as *mut $regs);

                    if self.invert {
                        data = !data;
                    }

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
