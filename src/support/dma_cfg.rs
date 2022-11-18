#[repr(u8)]
pub enum DMAWordSize {
    S8Bit = 0b00,
    S16Bit = 0b01,
    S32Bit = 0b10,
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum DMADir {
    Perith2mem = 0,
    Mem2Perith = 1,
}
