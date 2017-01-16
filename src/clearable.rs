/// Types that can safely be dropped after first being overwritten by zeros.
///
/// There is a default implementation for all `Copy+Default` types and all 
/// unsized arrays of `Clerable` types.  You need to implement `Clerable` 
/// yourself for unsized structs, or if your type could be `Copy+Default` 
/// but you do not want it to be.
///
/// ```
/// # use clear_on_drop::Clearable;
/// struct MyNonCopyType([u8; 128]);
/// unsafe impl Clearable for MyNonCopyType {
///     unsafe fn clear(&mut self) {
///         *self = ::std::mem::zeroed::<MyNonCopyType>();
///     }
/// }
/// ```
///
/// We need `Copy` to prevent pointer types like `&mut T` `Box<T>`,
/// `Arc<T>`, etc. from being `Clearable`.  We need `Default` bound only
/// because `Shared<T>` is `Copy`.  At present, `Shared<T>` is the only
/// "bad" `Copy` type, but future abstractions built using `Shared<T>`
/// might be `Copy`, including perhaps garbage collected abstractions.  
/// Using these could result in memory leaks or calling `drop` with a
/// null pointer.

pub unsafe trait Clearable {
    /// Clear data by dropping it and overwriting it with fixed data, 
    /// possibly leaving in an unusable state.  The object must be
    /// safe to drop again after being overwritten with zeros however.
    unsafe fn clear(&mut self);
}

unsafe impl<T> Clearable for T where T: Copy+Default {
    unsafe fn clear(&mut self) {
        *self = ::std::mem::zeroed::<Self>();
        // Assigning like this is equivelent to 
        //   ::std::ptr::drop_in_place::<Self>(self);
        //   ::std::ptr::write_unaligned::<T>(self, ::std::mem::zeroed::<Self>())
        // because the safety notes on ptr::read say it drops the value
        // previously at *self.
        ::std::ptr::write::<Self>(self, Default::default());
        // Should this be ::std::ptr::write_unaligned?
        // see https://github.com/rust-lang/rust/issues/37955
    }
}

unsafe impl<T> Clearable for [T] where T: Clearable {
    unsafe fn clear(&mut self) {
        for s in self.iter_mut() {
            *s = ::std::mem::zeroed::<T>();
        }
    }
}

