use core::marker::PhantomData;

use num::Integer;

use super::bus::Bus;

pub struct Offsets<T: num::Integer + Copy> {
    pub oe1: T,
    pub oe2: T,

    pub a: T,
    pub b: T,
    pub c: T,

    pub d: T,
    pub e: T,
    pub f: T,
}

pub struct CatodesSelector<T: num::Integer + Copy, B, const C: usize> {
    bus: B,
    offsets: Offsets<T>,
    _t: PhantomData<T>,
}

impl<B, const C: usize> CatodesSelector<u16, B, C>
where
    B: Bus<u16>,
{
    pub fn new(catodes_bus: B, offsets: Offsets<u16>) -> Self {
        Self {
            bus: catodes_bus,
            offsets,
            _t: PhantomData,
        }
    }

    pub fn disable(&mut self) {
        self.bus.write(0);
    }

    pub fn select_column(&self, col: u16) -> u16 {
        col
    }

    // OE1 - Четные
    // OE2 - нечетные
    // На схеме катоды протумерованы с 1, так что первый (нулевой) катод включается через OE2
    pub fn apply_column(&mut self, col: u16) {
        let (c, oe) = if col.is_even() {
            (col, self.offsets.oe2)
        } else {
            (C as u16 - col, self.offsets.oe1)
        };

        let abc = self.to_abc((c >> 1) & 0b111_u16);
        let def = self.to_def((c >> (1 + 3)) & 0b111_u16);

        self.bus.write(def | abc | oe)
    }

    fn to_abc(&self, bits: u16) -> u16 {
        let mut res = 0;
        if bits & (1 << 0) != 0 {
            res |= self.offsets.a;
        }
        if bits & (1 << 1) != 0 {
            res |= self.offsets.b;
        }
        if bits & (1 << 2) != 0 {
            res |= self.offsets.c;
        }
        res
    }

    fn to_def(&self, bits: u16) -> u16 {
        let mut res = 0;
        if bits & (1 << 0) != 0 {
            res |= self.offsets.d;
        }
        if bits & (1 << 1) != 0 {
            res |= self.offsets.e;
        }
        if bits & (1 << 2) != 0 {
            res |= self.offsets.f;
        }
        res
    }
}
