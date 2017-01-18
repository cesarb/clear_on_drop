//! Prevent agressive code removal optimizations.
//!
//! The functions in this module "hide" a variable from the optimizer,
//! so that it believes the variable has been read and/or modified in
//! unpredictable ways, while in fact nothing happened.
//!
//! Inspired by/based on Linux kernel's OPTIMIZER_HIDE_VAR, which in
//! turn was based on the earlier RELOC_HIDE macro.

/// Make the optimizer believe the memory pointed to by `ptr` is read
/// and modified arbitrarily.
#[inline]
pub fn hide_mem<T>(ptr: &mut T) where T: ?Sized {
    hide_mem_impl(ptr);
}

/// Make the optimizer believe the pointer returned by this function is
/// possibly unrelated (except for the lifetime) to `ptr`.
#[inline]
pub fn hide_ptr<P>(mut ptr: P) -> P {
    hide_mem::<P>(&mut ptr);
    ptr
}

#[cfg(feature = "nightly")]
use self::nightly::*;

#[cfg(not(feature = "no_cc"))]
use self::cc::*;

#[cfg(all(feature = "no_cc", not(feature = "nightly")))]
use self::fallback::*;

// On nightly, inline assembly can be used.
#[cfg(feature = "nightly")]
mod nightly {
    #[inline]
    pub fn hide_mem_impl<T>(ptr: *mut T) where T: ?Sized {
        unsafe {
            asm!("" : "=*m" (ptr) : "*0" (ptr));
        }
    }
}

// When a C compiler is available, a dummy C function can be used.
#[cfg(not(feature = "no_cc"))]
mod cc {
    use std::os::raw::c_void;

    extern "C" {
        fn clear_on_drop_hide(ptr: *mut c_void) -> *mut c_void;
    }

    #[inline]
    pub fn hide_mem_impl<T>(ptr: *mut T) where T: ?Sized {
        unsafe {
            clear_on_drop_hide(ptr as *mut c_void);
        }
    }
}

// When neither is available, pretend the pointer is sent to a thread,
// and hope this is enough to confuse the optimizer.
#[cfg(all(feature = "no_cc", not(feature = "nightly")))]
mod fallback {
    use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

    #[inline]
    pub fn hide_mem_impl<T>(ptr: *mut T) where T: ?Sized {
        static DUMMY: AtomicUsize = ATOMIC_USIZE_INIT;
        DUMMY.store(ptr as usize, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    struct Place {
        data: [u32; 4],
    }

    const DATA: [u32; 4] = [0x01234567, 0x89abcdef, 0xfedcba98, 0x76543210];

    #[test]
    fn hide_mem() {
        let mut place = Place { data: DATA };
        super::hide_mem(&mut place);
        assert_eq!(place.data, DATA);
    }

    #[test]
    fn hide_ptr() {
        let mut place = Place { data: DATA };
        let before = &mut place as *mut _;
        let after = super::hide_ptr(&mut place);
        assert_eq!(before, after as *mut _);
        assert_eq!(after.data, DATA);
    }

    #[test]
    fn hide_unsized() {
        let mut place = vec![1u64,2u64];
        super::hide_mem::<[u64]>(place.as_mut_slice());
        assert_eq!(place.as_slice(), &[1,2]);
    }
}
