#[macro_export]
macro_rules! parralel_port {
    ($name: ident, $port: ty, $parts: ty, $regs: ty, $valuetype:ty => ($([$pin:ident: $bit_n:expr]),+)) => {
        struct $name{
            regs_ptr: usize,
            mask: $valuetype,
        }

        impl $name {
            pub fn init(port: $port, rcc: &mut stm32f0xx_hal::rcc::Rcc) -> Self {
                let parts = port.split(rcc);
                let mut mask = 0;

                cortex_m::interrupt::free(|cs| {
                    $(
                        let _pin = parts.$pin.into_push_pull_output(cs);
                        mask |= 1 << ($bit_n);
                    )*
               });

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
