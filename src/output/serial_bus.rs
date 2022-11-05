pub struct SerialBus {}

impl SerialBus {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::bus::Bus<[u8]> for SerialBus {
    fn write(&mut self, data: &[u8]) {
        todo!()
    }
}
