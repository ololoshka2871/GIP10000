use super::bus::Bus;

pub struct AnodesDriver<B: Bus<[u8]>> {
    bus: B,
}

impl<B: Bus<[u8]>> AnodesDriver<B> {
    pub fn new(anodes_bus: B) -> Self {
        Self { bus: anodes_bus }
    }

    pub fn set_colum_pixels(pixels: &[u8]) {
        todo!()
    }
}
