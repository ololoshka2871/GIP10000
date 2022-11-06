use core::marker::PhantomData;

pub struct ParalelBus<T> {
    _t: PhantomData<T>,
}

impl<T> ParalelBus<T> {
    pub fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T> super::bus::Bus<T> for ParalelBus<T> {
    fn write(&mut self, data: T) {
        todo!()
    }
}
