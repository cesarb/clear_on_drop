#![allow(missing_docs)] // TODO

use hide::hide_mem;

pub trait Clear {
    fn clear(&mut self);
}

impl<T> Clear for T
    where T: Default
{
    #[inline]
    fn clear(&mut self) {
        *self = Default::default();
        hide_mem::<T>(self);
    }
}
