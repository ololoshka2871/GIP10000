use core::ops::Range;

pub struct StaticBufReader(pub &'static [u8]);

impl StaticBufReader {
    pub fn empty() -> Self {
        unsafe { StaticBufReader(core::slice::from_raw_parts(0x01 as *const u8, 0)) }
    }
}

impl From<Range<*const u8>> for StaticBufReader {
    fn from(range: Range<*const u8>) -> Self {
        unsafe { StaticBufReader(core::slice::from_ptr_range::<'static, _>(range)) }
    }
}

unsafe impl embedded_dma::ReadBuffer for StaticBufReader {
    type Word = u8;

    unsafe fn read_buffer(&self) -> (*const Self::Word, usize) {
        (self.0.as_ptr(), self.0.len())
    }
}
