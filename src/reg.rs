use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::access::{self, Access};
use crate::integers::Integer;

#[cfg(doc)]
use crate::access::{ReadOnly, ReadWrite, WriteOnly};
#[cfg(doc)]
use crate::RegMap;

/// A pointer to a register with volatile reads and writes.
///
/// # Access permissions
/// The read/write permission for the register is set by the generic parameter `A`:
/// - when `A` is [`ReadOnly`] or [`ReadWrite`], the register can be read from with [`Reg::read`],
/// - when `A` is [`WriteOnly`] or [`ReadWrite`], the register can be written to with
///   [`Reg::write`].
///
/// Access permissions are defined by the derive macro [`RegMap`] using the `#[reg()]` attribute,
/// see [Access permissions](crate#access-permissions) in the crate documentation.
pub struct Reg<'a, T, A> {
    ptr: NonNull<T>,
    _ref: PhantomData<&'a T>,
    _acs: PhantomData<A>,
}
impl<'a, T: Integer, A: Access> Reg<'a, T, A> {
    /// Creates a new `Reg`.
    ///
    /// ⚠️ This function is called by the field-access methods defined by the derive macro
    /// [`RegMap`]. Do *not* call this function directly. Changes to this function are not
    /// considered semver breaking.
    ///
    /// # Safety
    /// - `ptr` must be [valid for reads](core::ptr::read_volatile#safety) if `A: Readable`,
    /// - `ptr` must be [valid for writes](core::ptr::write_volatile#safety) if `A: Writable`,
    /// - `ptr` must be properly aligned;
    /// - `ptr` must be valid for the whole lifetime `'a`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[inline]
    pub const unsafe fn __MACRO_ONLY__from_ptr(ptr: *mut T) -> Self {
        Self::from_nonnull(NonNull::new_unchecked(ptr))
    }
    #[inline]
    pub(crate) const unsafe fn from_nonnull(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _ref: PhantomData,
            _acs: PhantomData,
        }
    }
    /// Returns a raw pointer to the underlying register.
    #[inline]
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
    /// Perform a volatile read.
    #[inline]
    pub fn read(&self) -> T
    where
        A: access::Readable,
    {
        unsafe { self.ptr.read_volatile() }
    }
    /// Perform a volatile write.
    #[inline]
    pub fn write(&self, val: T)
    where
        A: access::Writable,
    {
        unsafe { self.ptr.write_volatile(val) }
    }
}

/// Pointers to custom register maps derived by [`RegMap`].
///
/// ⚠️ This trait is implemented by the derive macro [`RegMap`]. Do *not* implement this trait
/// directly. Adding new required items to this trait is not considered semver breaking.
///
/// # Safety
/// This trait should only be implemented through the derive macro [`RegMap`].
pub unsafe trait RegMapPtr<'a>: Sized + 'a {
    type RegMap;

    /// Creates a new pointer to `Self::RegMap`.
    ///
    /// # Safety
    /// - `ptr` must point to a valid instance of `Self::RegMap`;
    /// - `ptr` must be valid for the whole lifetime `'a`;
    /// - all fields of `Self::RegMap` must allow volatile reads/writes.
    unsafe fn from_nonnull(ptr: NonNull<Self::RegMap>) -> Self;

    /// Creates a new pointer to `Self::RegMap`.
    ///
    /// # Safety
    /// - `ptr` must not be null;
    /// - `ptr` must point to a valid instance of `Self::RegMap`;
    /// - `ptr` must be valid for the whole lifetime `'a`;
    /// - all fields of `Self::RegMap` must allow volatile reads/writes.
    unsafe fn from_ptr(ptr: *mut Self::RegMap) -> Self;

    /// Return a pointer to `Self::RegMap` from a mutable (exclusive) reference.
    fn from_mut(reg: &'a mut Self::RegMap) -> Self;

    /// Returns a raw pointer to the underlying register map.
    fn as_ptr(&self) -> *mut Self::RegMap;
}
