use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use clear::Clear;

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
///     key.value = 0x01234567;
///     // ...
/// }   // key is dropped here
/// assert_eq!(place.value, 0);
/// ```
pub struct ClearOnDrop<P>
    where P: DerefMut,
          P::Target: Clear
{
    _place: P,
}

impl<P> ClearOnDrop<P>
    where P: DerefMut,
          P::Target: Clear
{
    /// Creates a new `ClearOnDrop` which clears `place` on drop.
    ///
    /// The `place` parameter can be a `&mut T`, a `Box<T>`, or other
    /// containers which behave like `Box<T>`.
    #[inline]
    pub fn new(place: P) -> Self {
        ClearOnDrop { _place: place }
    }

    /// Consumes the `ClearOnDrop`, returning the `place` without clearing.
    ///
    /// Note: this is an associated function, which means that you have
    /// to call it as `ClearOnDrop::into_uncleared_place(c)` instead of
    /// `c.into_uncleared_place()`. This is so that there is no conflict
    /// with a method on the inner type.
    #[inline]
    pub fn into_uncleared_place(c: Self) -> P {
        unsafe {
            let place = ptr::read(&c._place);
            mem::forget(c);
            place
        }
    }

    /// Consumes the `ClearOnDrop`, returning the `place` after clearing.
    ///
    /// Note: this is an associated function, which means that you have
    /// to call it as `ClearOnDrop::into_place(c)` instead of
    /// `c.into_place()`. This is so that there is no conflict with a
    /// method on the inner type.
    #[inline]
    pub fn into_place(mut c: Self) -> P {
        c.clear();
        Self::into_uncleared_place(c)
    }
}

impl<P> fmt::Debug for ClearOnDrop<P>
    where P: DerefMut + fmt::Debug,
          P::Target: Clear
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self._place, f)
    }
}

impl<P> Deref for ClearOnDrop<P>
    where P: DerefMut,
          P::Target: Clear
{
    type Target = P::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Deref::deref(&self._place)
    }
}

impl<P> DerefMut for ClearOnDrop<P>
    where P: DerefMut,
          P::Target: Clear
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        DerefMut::deref_mut(&mut self._place)
    }
}

impl<P> Drop for ClearOnDrop<P>
    where P: DerefMut,
          P::Target: Clear
{
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

// std::convert traits

impl<P, T: ?Sized> AsRef<T> for ClearOnDrop<P>
    where P: DerefMut + AsRef<T>,
          P::Target: Clear
{
    #[inline]
    fn as_ref(&self) -> &T {
        AsRef::as_ref(&self._place)
    }
}

impl<P, T: ?Sized> AsMut<T> for ClearOnDrop<P>
    where P: DerefMut + AsMut<T>,
          P::Target: Clear
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        AsMut::as_mut(&mut self._place)
    }
}

// std::hash traits

impl<P> Hash for ClearOnDrop<P>
    where P: DerefMut + Hash,
          P::Target: Clear
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self._place, state)
    }
}

// std::cmp traits

impl<P, Q> PartialEq<ClearOnDrop<Q>> for ClearOnDrop<P>
    where P: DerefMut + PartialEq<Q>,
          P::Target: Clear,
          Q: DerefMut,
          Q::Target: Clear
{
    #[inline]
    fn eq(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialEq::eq(&self._place, &other._place)
    }

    #[inline]
    fn ne(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialEq::ne(&self._place, &other._place)
    }
}

impl<P> Eq for ClearOnDrop<P>
    where P: DerefMut + Eq,
          P::Target: Clear
{
}

impl<P, Q> PartialOrd<ClearOnDrop<Q>> for ClearOnDrop<P>
    where P: DerefMut + PartialOrd<Q>,
          P::Target: Clear,
          Q: DerefMut,
          Q::Target: Clear
{
    #[inline]
    fn partial_cmp(&self, other: &ClearOnDrop<Q>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self._place, &other._place)
    }

    #[inline]
    fn lt(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::lt(&self._place, &other._place)
    }

    #[inline]
    fn le(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::le(&self._place, &other._place)
    }

    #[inline]
    fn gt(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::gt(&self._place, &other._place)
    }

    #[inline]
    fn ge(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::ge(&self._place, &other._place)
    }
}

impl<P> Ord for ClearOnDrop<P>
    where P: DerefMut + Ord,
          P::Target: Clear
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self._place, &other._place)
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
        assert_eq!(place.data, [0; 4]);
    }

    #[test]
    fn on_box() {
        let place: Box<Place> = Box::new(Default::default());
        let mut clear = ClearOnDrop::new(place);
        clear.data = DATA;
        assert_eq!(clear.data, DATA);
    }

    #[test]
    fn into_box() {
        let place: Box<Place> = Box::new(Default::default());
        let mut clear = ClearOnDrop::new(place);
        clear.data = DATA;
        assert_eq!(clear.data, DATA);

        let place = ClearOnDrop::into_place(clear);
        assert_eq!(place.data, [0; 4]);
    }

    #[test]
    fn into_uncleared_box() {
        let place: Box<Place> = Box::new(Default::default());
        let mut clear = ClearOnDrop::new(place);
        clear.data = DATA;
        assert_eq!(clear.data, DATA);

        let place = ClearOnDrop::into_uncleared_place(clear);
        assert_eq!(place.data, DATA);
    }

    #[test]
    fn on_slice() {
        let mut place: [u32; 4] = Default::default();
        {
            let mut clear = ClearOnDrop::new(&mut place[..]);
            clear.copy_from_slice(&DATA);
            assert_eq!(&clear[..], DATA);
        }
        assert_eq!(place, [0; 4]);
    }

    #[test]
    fn on_boxed_slice() {
        let place: Box<[u32]> = vec![0; 4].into_boxed_slice();
        let mut clear = ClearOnDrop::new(place);
        clear.copy_from_slice(&DATA);
        assert_eq!(&clear[..], DATA);
    }
}
