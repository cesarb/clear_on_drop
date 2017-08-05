//! Traits to completely overwrite a value, without leaking data.
//!
//! # Examples
//!
//! Basic use:
//!
//! ```
//! # use clear_on_drop::clear::Clear;
//! #[derive(Default)]
//! struct MyData {
//!     value: u32,
//! }
//!
//! let mut place = MyData { value: 0x01234567 };
//! place.clear();
//! assert_eq!(place.value, 0);
//! ```
//!
//! Showing no data is leaked:
//!
//! ```
//! # use std::mem;
//! # use std::slice;
//! # use clear_on_drop::clear::Clear;
//! #[derive(Default)]
//! struct MyData {
//!     value: Option<u32>,
//! }
//!
//! let mut place = MyData { value: Some(0x41414141) };
//! place.clear();
//! assert_eq!(place.value, None);
//!
//! fn as_bytes<T>(x: &T) -> &[u8] {
//!     unsafe {
//!         slice::from_raw_parts(x as *const T as *const u8, mem::size_of_val(x))
//!     }
//! }
//! assert!(!as_bytes(&place).contains(&0x41));
//! ```

use core::mem;
use core::ptr;

use hide::hide_mem_impl;

/// An operation to completely overwrite a value, without leaking data.
///
/// Do not implement this trait; implement `InitializableFromZeroed`
/// instead. This trait's blanket implementation uses several tricks to
/// make sure no data is leaked.
pub trait Clear {
    /// Completely overwrites this value.
    fn clear(&mut self);
}

impl<T: ?Sized> Clear for T
    where T: InitializableFromZeroed
{
    #[inline]
    fn clear(&mut self) {
        let size = mem::size_of_val(self);
        unsafe {
            let ptr = self as *mut Self;
            ptr::drop_in_place(ptr);
            ptr::write_bytes(ptr as *mut u8, 0, size);
            hide_mem_impl::<Self>(ptr);
            Self::initialize(ptr);
        }
    }
}

/// A type that can be initialized to a valid value, after being set to
/// all-bits-zero.
pub trait InitializableFromZeroed {
    /// Called to initialize a place to a valid value, after it is set
    /// to all-bits-zero.
    ///
    /// If all-bits-zero is a valid value for a place, this method can
    /// be left empty.
    unsafe fn initialize(place: *mut Self);
}

impl<T> InitializableFromZeroed for T
    where T: Default
{
    #[inline]
    unsafe fn initialize(place: *mut Self) {
        ptr::write(place, Default::default());
    }
}

impl<T> InitializableFromZeroed for [T]
    where T: ZeroSafe
{
    #[inline]
    unsafe fn initialize(_place: *mut Self) {}
}

/// Unsafe trait to indicate which types are safe to set to all-bits-zero.
pub unsafe trait ZeroSafe {}

// Yes, this is core::nonzero::Zeroable
unsafe impl<T: ?Sized> ZeroSafe for *const T {}
unsafe impl<T: ?Sized> ZeroSafe for *mut T {}
unsafe impl ZeroSafe for isize {}
unsafe impl ZeroSafe for usize {}
unsafe impl ZeroSafe for i8 {}
unsafe impl ZeroSafe for u8 {}
unsafe impl ZeroSafe for i16 {}
unsafe impl ZeroSafe for u16 {}
unsafe impl ZeroSafe for i32 {}
unsafe impl ZeroSafe for u32 {}
unsafe impl ZeroSafe for i64 {}
unsafe impl ZeroSafe for u64 {}
#[cfg(feature = "nightly")]
unsafe impl ZeroSafe for i128 {}
#[cfg(feature = "nightly")]
unsafe impl ZeroSafe for u128 {}
