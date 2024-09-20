use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::access::Access;
use crate::bounds;
use crate::integers::Integer;
use crate::iter;
use crate::reg::{Reg, RegMapPtr};

#[cfg(doc)]
use crate::RegMap;

/// An array of registers.
///
/// Element type can be:
/// - a basic register of type [`Reg`];
/// - a custom register map (`struct`) implementing the trait [`RegMapPtr`] through the derive
///   macro [`RegMap`];
/// - another `RegArray` (multidimensional array).
pub struct RegArray<'a, P: ArrayElem<'a>, const N: usize> {
    ptr: NonNull<[P::Target; N]>,
    _ref: PhantomData<&'a [P::Target; N]>,
}
impl<'a, P: ArrayElem<'a>, const N: usize> RegArray<'a, P, N> {
    /// Creates a new `RegArray`.
    ///
    /// ⚠️ This function is called by the field-access methods defined by the derive macro
    /// [`RegMap`](crate::RegMap). Do *not* call this function directly. Changes to this function
    /// are not considered semver breaking.
    ///
    /// # Safety
    /// - `ptr` must be properly aligned;
    /// - `ptr` must point to `N` contiguous elements of type `P::Target`,
    /// - `ptr` must be valid for the whole lifetime `'a`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[inline]
    pub const unsafe fn __MACRO_ONLY__from_ptr(ptr: *mut [P::Target; N]) -> Self {
        Self::from_nonnull(NonNull::new_unchecked(ptr))
    }
    #[inline]
    const unsafe fn from_nonnull(ptr: NonNull<[P::Target; N]>) -> Self {
        Self {
            ptr,
            _ref: PhantomData,
        }
    }
    /// Returns a raw pointer to the underlying pointer array.
    #[inline]
    pub const fn as_ptr(&self) -> *mut [P::Target; N] {
        self.ptr.as_ptr()
    }
    /// Returns the number of pointers in the array.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub const fn len(&self) -> usize {
        N
    }
    /// Access the pointer at `index`.
    ///
    /// # Panics
    /// If `index` is out of bounds, i.e. if `index >= N`.
    #[inline]
    pub fn idx(&self, index: usize) -> P {
        bounds::check_index::<N>(index);
        // SAFETY: we checked i is in bounds
        unsafe { self.idx_unchecked(index) }
    }
    /// Access the pointer at `index`, without doing bounds checking.
    ///
    /// # Safety
    /// `index` must be in bounds: `index < N`.
    #[inline]
    pub unsafe fn idx_unchecked(&self, index: usize) -> P {
        let base: NonNull<P::Target> = self.ptr.cast();
        // SAFETY: the caller promises we are in bounds
        unsafe { P::from_nonnull(base.add(index)) }
    }
    /// Returns an iterator over the pointer array.
    pub fn iter(
        &self,
    ) -> impl 'a + ExactSizeIterator<Item = P> + DoubleEndedIterator + FusedIterator + Clone {
        iter::RegArrayIter::new(self.ptr)
    }
    /// Returns an iterator over a subslice `[start..end]` of the pointer array.
    ///
    /// # Panics
    /// If `[start..end]` is out of bounds.
    pub fn iter_slice(
        &self,
        start: usize,
        end: usize,
    ) -> impl 'a + ExactSizeIterator<Item = P> + DoubleEndedIterator + FusedIterator + Clone {
        bounds::check_slice::<N>(start, end);
        let base: NonNull<P::Target> = self.ptr.cast();
        // SAFETY: we checked start..end is in bounds
        unsafe {
            let slice = NonNull::slice_from_raw_parts(base.add(start), end - start);
            iter::RegArrayIter::new(slice)
        }
    }
}

/// Types that can be stored in a [`RegArray`].
///
/// ⚠️ This trait is sealed and cannot be implemented for types outside of this crate.
pub trait ArrayElem<'a>: 'a + private::Sealed {
    /// The target type of the pointer stored in the array.
    type Target;

    /// Creates a new pointer to `Self::Target`.
    ///
    /// # Safety
    /// - `ptr` must point to a valid instance of `Self::Target`;
    /// - `ptr` must be valid for the whole lifetime `'a`.
    unsafe fn from_nonnull(ptr: NonNull<Self::Target>) -> Self;
}

// arrays of basic registers
impl<'a, T: Integer, A: Access> ArrayElem<'a> for Reg<'a, T, A> {
    type Target = T;

    unsafe fn from_nonnull(ptr: NonNull<Self::Target>) -> Self {
        Reg::from_nonnull(ptr)
    }
}

// arrays of custom register maps (structs)
impl<'a, T: RegMapPtr<'a>> ArrayElem<'a> for T {
    type Target = T::RegMap;

    unsafe fn from_nonnull(ptr: NonNull<Self::Target>) -> Self {
        T::from_nonnull(ptr)
    }
}

// multidimensional arrays
impl<'a, T: ArrayElem<'a>, const N: usize> ArrayElem<'a> for RegArray<'a, T, N> {
    type Target = [T::Target; N];

    unsafe fn from_nonnull(ptr: NonNull<Self::Target>) -> Self {
        RegArray::from_nonnull(ptr)
    }
}

mod private {
    use crate::access::Access;
    use crate::arr::{ArrayElem, RegArray};
    use crate::integers::Integer;
    use crate::reg::{Reg, RegMapPtr};

    pub trait Sealed {}
    impl<'a, T: Integer, A: Access> Sealed for Reg<'a, T, A> {}
    impl<'a, T: RegMapPtr<'a>> Sealed for T {}
    impl<'a, T: ArrayElem<'a>, const N: usize> Sealed for RegArray<'a, T, N> {}
}
