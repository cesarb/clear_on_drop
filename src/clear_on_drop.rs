use std::fmt;
use std::ops::{Deref, DerefMut};

use hide::hide_mem;

/// Zeroizes a storage location when dropped.
///
/// This struct contains a reference to a memory location, either as a
/// mutable borrow (`&mut T`), or as a owned container (`Box<T>` or
/// similar). When this struct is dropped, the referenced location is
/// overwritten with its `Default` value.
///
/// # Example
///
/// ```
/// # use clear_on_drop::ClearOnDrop;
/// #[derive(Default)]
/// struct MyData {
///     value: u32,
/// }
///
/// let mut place = MyData { value: 0 };
/// {
///     let mut key = ClearOnDrop::new(&mut place);
///     key.value = 0x012345678;
///     // ...
/// }   // key is dropped here
/// assert_eq!(place.value, 0);
/// ```

pub struct ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut
{
    _place: P,
}

impl<T, P> ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut
{
    /// Creates a new `ClearOnDrop` which clears `place` on drop.
    ///
    /// The `place` parameter can be a `&mut T`, a `Box<T>`, or other
    /// containers which behave like `Box<T>`.
    #[inline]
    pub fn new(place: P) -> Self {
        ClearOnDrop { _place: place }
    }
}

impl<T, P> fmt::Debug for ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut + fmt::Debug
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self._place, f)
    }
}

impl<T, P> Deref for ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Deref::deref(&self._place)
    }
}

impl<T, P> DerefMut for ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        DerefMut::deref_mut(&mut self._place)
    }
}

impl<T, P> Drop for ClearOnDrop<T, P>
    where T: Default,
          P: Deref<Target = T> + DerefMut
{
    #[inline]
    fn drop(&mut self) {
        let place = self.deref_mut();
        *place = Default::default();
        hide_mem::<T>(place);
    }
}

#[cfg(test)]
mod tests {
    use super::ClearOnDrop;

    #[derive(Debug, Default)]
    struct Place {
        data: [u32; 4],
    }

    const DATA: [u32; 4] = [0x01234567, 0x89abcdef, 0xfedcba98, 0x76543210];

    #[test]
    fn on_stack() {
        let mut place: Place = Default::default();
        {
            let mut clear = ClearOnDrop::new(&mut place);
            clear.data = DATA;
            assert_eq!(clear.data, DATA);
        }
        assert_eq!(place.data, [0, 0, 0, 0]);
    }

    #[test]
    fn on_box() {
        let place: Box<Place> = Box::new(Default::default());
        let mut clear = ClearOnDrop::new(place);
        clear.data = DATA;
        assert_eq!(clear.data, DATA);
    }
}
