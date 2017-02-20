use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ops::{Index,IndexMut};
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

impl<P> Clone for ClearOnDrop<P>
    where P: DerefMut + Clone,
          P::Target: Clear
{
    #[inline]
    fn clone(&self) -> Self {
        ClearOnDrop { _place: Clone::clone(&self._place) }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        Clone::clone_from(&mut self._place, &source._place)
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

// std::borrow traits

// The `T: Clear` bound avoids a conflict with the blanket impls
// `impl<T> Borrow<T> for T` and `impl<T> BorrowMut<T> for T`, since
// `ClearOnDrop<_>` is not `Clear`.

impl<P, T: ?Sized> Borrow<T> for ClearOnDrop<P>
    where P: DerefMut + Borrow<T>,
          P::Target: Clear,
          T: Clear
{
    #[inline]
    fn borrow(&self) -> &T {
        Borrow::borrow(&self._place)
    }
}

impl<P, T: ?Sized> BorrowMut<T> for ClearOnDrop<P>
    where P: DerefMut + BorrowMut<T>,
          P::Target: Clear,
          T: Clear
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        BorrowMut::borrow_mut(&mut self._place)
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

// more std::ops traits

impl<I,P> Index<I> for ClearOnDrop<P>
    where P: DerefMut + Index<I>,
          P::Target: Clear
{
    type Output = P::Output;

    fn index(&self, i: I) -> &Self::Output {
        &self._place[i]
    }
}

impl<I,P> IndexMut<I> for ClearOnDrop<P>
    where P: DerefMut + IndexMut<I>,
          P::Target: Clear
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        &mut self._place[i]
    }
}

macro_rules! delegate_ops {
    ($trait_assign:ident, $fn_assign:ident, $trait_binop:ident, $fn_binop:ident) => (
        use std::ops::{$trait_assign,$trait_binop};

/*
        // This works but seems dangerous because it can take any sort of R.
        impl<P,R> $trait_assign<R> for ClearOnDrop<P>
            where P: DerefMut + $trait_assign<R>,
                  P::Target: Clear
        {
            #[inline]
            fn $fn_assign(&mut self, other: R) {
                $trait_assign::$fn_assign(&mut self._place,other)
            }
        }
*/

        // This seems safe because everything is forced ot be a reference.
        impl<'r,P,R> $trait_assign<&'r R> for ClearOnDrop<P>
            where P: DerefMut + $trait_assign<&'r R>,
                  P::Target: Clear,
                  R: 'r
        {
            #[inline]
            fn $fn_assign(&mut self, other: &'r R) {
                $trait_assign::$fn_assign(&mut self._place,other)
            }
        }

/*
        // This seems safe because afaik ClearOnDrop is always a reference,
        // but it does not work currently.
        impl<'r,P,R> $trait_assign<ClearOnDrop<R>> for ClearOnDrop<P>
            where P: DerefMut + $trait_assign<&'r R> + 'r,
                  P::Target: Clear,
                  R: DerefMut + 'r,
                  R::Target: Clear,
        {
            #[inline]
            fn $fn_assign(&mut self, other: ClearOnDrop<R>) {
                $trait_assign::$fn_assign(&mut self._place,&other._place)
            }
        }
*/

        // This works and keeps its arguments safe by only taking references,
        // but Output is returned by value, which sounds risky. 
        impl<'p,'r,P,R> $trait_binop<&'r R> for &'p ClearOnDrop<P>
            where P: DerefMut + 'p,
                  &'p P: $trait_binop<&'r R>,
                  P::Target: Clear
        {
            type Output = <&'p P as $trait_binop<&'r R>>::Output;

            #[inline]
            fn $fn_binop(self, other: &'r R) -> Self::Output {
                $trait_binop::$fn_binop(&self._place,other) 
            }
        }

/*
        // This works but seems dangerous because it can take any sort of R.
        impl<'p,P,R> $trait_binop<R> for &'p ClearOnDrop<P>
            where P: DerefMut + 'p,
                  &'p P: $trait_binop<R>,
                  P::Target: Clear
        {
            type Output = <&'p P as $trait_binop<R>>::Output;

            #[inline]
            fn $fn_binop(self, other: R) -> Self::Output {
                $trait_binop::$fn_binop(&self._place,other) 
            }
        }
*/
    )
}

delegate_ops!(AddAssign,add_assign,Add,add);
delegate_ops!(SubAssign,sub_assign,Sub,sub);
delegate_ops!(MulAssign,mul_assign,Mul,mul);
delegate_ops!(DivAssign,div_assign,Div,div);
delegate_ops!(RemAssign,rem_assign,Rem,rem);

delegate_ops!(BitAndAssign,bitand_assign,BitAnd,bitand);
delegate_ops!(BitXorAssign,bitxor_assign,BitXor,bitxor);
delegate_ops!(BitOrAssign,bitor_assign,BitOr,bitor);

delegate_ops!(ShlAssign,shl_assign,Shl,shl);
delegate_ops!(ShrAssign,shr_assign,Shr,shr);

// TODO: Neg Not

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
