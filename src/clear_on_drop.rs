use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::{fmt, mem};

use crate::clear::Clear;

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
where
    P: DerefMut,
    P::Target: Clear,
{
    _place: ManuallyDrop<P>,
}

impl<P> ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    /// Creates a new `ClearOnDrop` which clears `place` on drop.
    ///
    /// The `place` parameter can be a `&mut T`, a `Box<T>`, or other
    /// containers which behave like `Box<T>`.
    ///
    /// Note: only the first level of dereference will be cleared. Do
    /// not use `&mut Box<T>` or similar as the place, since the heap
    /// contents won't be cleared in that case. If you need the place
    /// back, use `ClearOnDrop::into_place(...)` instead of a borrow.
    #[inline]
    pub fn new(place: P) -> Self {
        ClearOnDrop {
            _place: ManuallyDrop::new(place),
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

    /// Consumes the `ClearOnDrop`, returning the `place` without clearing.
    ///
    /// Note: this is an associated function, which means that you have
    /// to call it as `ClearOnDrop::into_uncleared_place(c)` instead of
    /// `c.into_uncleared_place()`. This is so that there is no conflict
    /// with a method on the inner type.
    #[inline]
    pub fn into_uncleared_place(mut c: Self) -> P {
        unsafe {
            let place = ptr::read(&c._place);
            ptr::write_bytes(
                &mut c._place as *mut _ as *mut u8,
                0,
                mem::size_of::<ManuallyDrop<P>>(),
            );
            ManuallyDrop::into_inner(place)
        }
    }
}

impl<P> Clone for ClearOnDrop<P>
where
    P: DerefMut + Clone,
    P::Target: Clear,
{
    #[inline]
    fn clone(&self) -> Self {
        ClearOnDrop {
            _place: Clone::clone(&self._place),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        Clone::clone_from(&mut self._place, &source._place)
    }
}

impl<P> fmt::Debug for ClearOnDrop<P>
where
    P: DerefMut + fmt::Debug,
    P::Target: Clear,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self._place, f)
    }
}

impl<P> Deref for ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    type Target = P::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Deref::deref(&self._place as &P)
    }
}

impl<P> DerefMut for ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        DerefMut::deref_mut(&mut self._place as &mut P)
    }
}

impl<P> Drop for ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    #[inline]
    fn drop(&mut self) {
        let ptr = &mut self._place as *mut _ as *mut u8;
        unsafe {
            if (0..mem::size_of::<ManuallyDrop<P>>() as isize)
                .fold(0, |acc, i| acc + *ptr.offset(i) as i32)
                != 0
            {
                self.clear();
                ManuallyDrop::drop(&mut self._place);
            }
        }
    }
}

// core::convert traits

impl<P, T: ?Sized> AsRef<T> for ClearOnDrop<P>
where
    P: DerefMut + AsRef<T>,
    P::Target: Clear,
{
    #[inline]
    fn as_ref(&self) -> &T {
        AsRef::as_ref(&self._place as &P)
    }
}

impl<P, T: ?Sized> AsMut<T> for ClearOnDrop<P>
where
    P: DerefMut + AsMut<T>,
    P::Target: Clear,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        AsMut::as_mut(&mut self._place as &mut P)
    }
}

// core::borrow traits

// The `T: Clear` bound avoids a conflict with the blanket impls
// `impl<T> Borrow<T> for T` and `impl<T> BorrowMut<T> for T`, since
// `ClearOnDrop<_>` is not `Clear`.

impl<P, T: ?Sized> Borrow<T> for ClearOnDrop<P>
where
    P: DerefMut + Borrow<T>,
    P::Target: Clear,
    T: Clear,
{
    #[inline]
    fn borrow(&self) -> &T {
        Borrow::borrow(&self._place as &P)
    }
}

impl<P, T: ?Sized> BorrowMut<T> for ClearOnDrop<P>
where
    P: DerefMut + BorrowMut<T>,
    P::Target: Clear,
    T: Clear,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        BorrowMut::borrow_mut(&mut self._place as &mut P)
    }
}

// core::hash traits

impl<P> Hash for ClearOnDrop<P>
where
    P: DerefMut + Hash,
    P::Target: Clear,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self._place as &P, state)
    }
}

// core::cmp traits

impl<P, Q> PartialEq<ClearOnDrop<Q>> for ClearOnDrop<P>
where
    P: DerefMut + PartialEq<Q>,
    P::Target: Clear,
    Q: DerefMut,
    Q::Target: Clear,
{
    #[inline]
    fn eq(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialEq::eq(&self._place as &P, &other._place as &Q)
    }

    #[inline]
    fn ne(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialEq::ne(&self._place as &P, &other._place as &Q)
    }
}

impl<P> Eq for ClearOnDrop<P>
where
    P: DerefMut + Eq,
    P::Target: Clear,
{
}

impl<P, Q> PartialOrd<ClearOnDrop<Q>> for ClearOnDrop<P>
where
    P: DerefMut + PartialOrd<Q>,
    P::Target: Clear,
    Q: DerefMut,
    Q::Target: Clear,
{
    #[inline]
    fn partial_cmp(&self, other: &ClearOnDrop<Q>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self._place as &P, &other._place as &Q)
    }

    #[inline]
    fn lt(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::lt(&self._place as &P, &other._place as &Q)
    }

    #[inline]
    fn le(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::le(&self._place as &P, &other._place as &Q)
    }

    #[inline]
    fn gt(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::gt(&self._place as &P, &other._place as &Q)
    }

    #[inline]
    fn ge(&self, other: &ClearOnDrop<Q>) -> bool {
        PartialOrd::ge(&self._place as &P, &other._place as &Q)
    }
}

impl<P> Ord for ClearOnDrop<P>
where
    P: DerefMut + Ord,
    P::Target: Clear,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self._place as &P, &other._place as &P)
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

    #[cfg(not(miri))]
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
    fn on_fixed_size_array() {
        let mut place: [u32; 4] = Default::default();
        {
            let mut clear = ClearOnDrop::new(&mut place);
            clear.copy_from_slice(&DATA);
            assert_eq!(&clear[..], DATA);
        }
        assert_eq!(place, [0; 4]);
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

    #[test]
    fn on_str_slice() {
        let mut place: Box<str> = "test".into();
        {
            let clear = ClearOnDrop::new(&mut place[..]);
            assert_eq!(&clear[..], "test");
        }
        assert_eq!(&place[..], "\x00\x00\x00\x00");
    }

    #[test]
    fn on_string() {
        let place: String = "test".into();
        let clear = ClearOnDrop::new(place);
        assert_eq!(&clear[..], "test");
    }

    #[cfg(not(miri))]
    #[test]
    fn into_string() {
        let place: String = "test".into();
        let ptr = place.as_ptr();

        let clear = ClearOnDrop::new(place);
        assert_eq!(&clear[..], "test");

        let place = ClearOnDrop::into_place(clear);
        assert_eq!(place, "\x00\x00\x00\x00");
        assert_eq!(place.as_ptr(), ptr);
    }

    #[test]
    fn into_uncleared_string() {
        let place: String = "test".into();
        let ptr = place.as_ptr();

        let clear = ClearOnDrop::new(place);
        assert_eq!(&clear[..], "test");

        let place = ClearOnDrop::into_uncleared_place(clear);
        assert_eq!(place, "test");
        assert_eq!(place.as_ptr(), ptr);
    }
}
