use core::marker::PhantomData;

use super::bus::Bus;

pub struct CatodesSelector<T, B: Bus<T>> {
    bus: B,
    _t: PhantomData<T>,
}

impl<T, B: Bus<T>> CatodesSelector<T, B> {
    pub fn new(catodes_bus: B) -> Self {
        Self { bus: catodes_bus, _t: PhantomData }
    }

    pub fn disable(&mut self) {
        todo!()
    }

    pub fn select_column(&self, col: T) -> T {
        col
    }

    pub fn enable_with(&mut self, col: T) {
        todo!()
    }
}
