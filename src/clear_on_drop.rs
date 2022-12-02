use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::mem::{size_of_val, MaybeUninit};
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
    _place: MaybeUninit<P>,
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
            _place: MaybeUninit::new(place),
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
            ptr::write(&mut c._place, MaybeUninit::zeroed());
            place.assume_init()
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
            _place: MaybeUninit::new(unsafe { Clone::clone(&self._place.assume_init_ref()) }),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        unsafe {
            Clone::clone_from(
                &mut self._place.assume_init_ref(),
                &source._place.assume_init_ref(),
            )
        }
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
        unsafe { Deref::deref(self._place.assume_init_ref()) }
    }
}

impl<P> DerefMut for ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { DerefMut::deref_mut(self._place.assume_init_mut()) }
    }
}

impl<P> Drop for ClearOnDrop<P>
where
    P: DerefMut,
    P::Target: Clear,
{
    #[inline]
    fn drop(&mut self) {
        let ptr = self._place.as_ptr() as *mut u8;
        unsafe {
            if (0..mem::size_of::<MaybeUninit<P>>() as isize)
                .fold(0, |acc, i| acc + *ptr.offset(i) as i32)
                != 0
            {
                self.clear();
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
        unsafe { AsRef::as_ref(self._place.assume_init_ref()) }
    }
}

impl<P, T: ?Sized> AsMut<T> for ClearOnDrop<P>
where
    P: DerefMut + AsMut<T>,
    P::Target: Clear,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        unsafe { AsMut::as_mut(self._place.assume_init_mut()) }
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
        unsafe { Borrow::borrow(self._place.assume_init_ref()) }
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
        unsafe { BorrowMut::borrow_mut(self._place.assume_init_mut()) }
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
        unsafe { Hash::hash(self._place.assume_init_ref(), state) }
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
        unsafe {
            PartialEq::eq(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }

    #[inline]
    fn ne(&self, other: &ClearOnDrop<Q>) -> bool {
        unsafe {
            PartialEq::ne(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
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
        unsafe {
            PartialOrd::partial_cmp(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }

    #[inline]
    fn lt(&self, other: &ClearOnDrop<Q>) -> bool {
        unsafe {
            PartialOrd::lt(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }

    #[inline]
    fn le(&self, other: &ClearOnDrop<Q>) -> bool {
        unsafe {
            PartialOrd::le(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }

    #[inline]
    fn gt(&self, other: &ClearOnDrop<Q>) -> bool {
        unsafe {
            PartialOrd::gt(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }

    #[inline]
    fn ge(&self, other: &ClearOnDrop<Q>) -> bool {
        unsafe {
            PartialOrd::ge(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
    }
}

impl<P> Ord for ClearOnDrop<P>
where
    P: DerefMut + Ord,
    P::Target: Clear,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe {
            Ord::cmp(
                &self._place.assume_init_ref(),
                &other._place.assume_init_ref(),
            )
        }
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
