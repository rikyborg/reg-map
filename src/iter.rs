//! Iterators over arrays of regsiters and pointers.

use core::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator};
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::arr::ArrayElem;

// most of the implementation is adapted from that of core::slice::Iter in Rust 1.80.1
// with adaptations for stable toolchain
// https://github.com/rust-lang/rust/blob/3f5fd8dd41153bc5fdca9427e9e05be2c767ba23/library/core/src/slice/iter/macros.rs

/// Iterator over a pointer array.
///
/// This struct is created by the [`iter`](crate::reg::PtrArray::iter) and
/// [`iter_slice`](crate::reg::PtrArray::iter_slice) methods on [`PtrArray`](crate::reg::PtrArray).
pub struct RegArrayIter<'a, P: ArrayElem<'a>> {
    start: NonNull<P::Target>,
    end: NonNull<P::Target>,
    _phantom: PhantomData<&'a ()>,
}
impl<'a, P: ArrayElem<'a>> Clone for RegArrayIter<'a, P> {
    fn clone(&self) -> Self {
        RegArrayIter {
            start: Clone::clone(&self.start),
            end: Clone::clone(&self.end),
            _phantom: Clone::clone(&self._phantom),
        }
    }
}

impl<'a, P: ArrayElem<'a>> RegArrayIter<'a, P> {
    pub(crate) const fn new(base: NonNull<[P::Target]>) -> Self {
        let start: NonNull<P::Target> = base.cast();
        let len = base.len();
        let end: NonNull<P::Target> = unsafe { start.add(len) };
        Self {
            start,
            end,
            _phantom: PhantomData,
        }
    }

    /// Helper function for moving the start of the iterator forwards by `offset` elements,
    /// returning the old start.
    ///
    /// # Safety
    ///
    /// The offset must not exceed `self.len()`.
    #[inline(always)]
    unsafe fn post_inc_start(&mut self, offset: usize) -> NonNull<P::Target> {
        let old = self.start;
        // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`
        unsafe { self.start = self.start.add(offset) };
        old
    }

    /// Helper function for moving the end of the iterator backwards by `offset` elements,
    /// returning the new end.
    ///
    /// # Safety
    ///
    /// The offset must not exceed `self.len()`.
    #[inline(always)]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> NonNull<P::Target> {
        // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`
        unsafe {
            self.end = self.end.sub(offset);
            self.end
        }
    }

    /// Returns the first element and moves the start of the iterator forwards by 1.
    ///
    /// # Safety
    ///
    /// The iterator must not be empty
    #[inline]
    unsafe fn next_unchecked(&mut self) -> P {
        // SAFETY: The caller promised there's at least one more item.
        unsafe { P::from_nonnull(self.post_inc_start(1)) }
    }

    /// Returns the last element and moves the end of the iterator backwards by 1.
    ///
    /// # Safety
    ///
    /// The iterator must not be empty
    #[inline]
    unsafe fn next_back_unchecked(&mut self) -> P {
        // SAFETY: The caller promised there's at least one more item.
        unsafe { P::from_nonnull(self.pre_dec_end(1)) }
    }
}

impl<'a, P: ArrayElem<'a>> Iterator for RegArrayIter<'a, P> {
    type Item = P;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: calling `next_unchecked` is safe since we check len() first.
        unsafe {
            if self.len() == 0 {
                None
            } else {
                Some(self.next_unchecked())
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.start = self.end;
            None
        } else {
            // SAFETY: We are in bounds.
            unsafe {
                self.post_inc_start(n);
                Some(self.next_unchecked())
            }
        }
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, P: ArrayElem<'a>> ExactSizeIterator for RegArrayIter<'a, P> {
    #[inline(always)]
    fn len(&self) -> usize {
        // SAFETY: by the type invariant pointers:
        // - are aligned
        // - inside the allocation
        // - `start <= end`

        // this is what core/slice/iter does, but it's unstable
        // https://github.com/rust-lang/rust/issues/95892
        // unsafe { self.end.sub_ptr(self.start) }

        // this is what the docs suggest as equivalent, but the codegen is less optimal
        // https://doc.rust-lang.org/core/primitive.pointer.html#method.sub_ptr
        unsafe { usize::try_from(self.end.offset_from(self.start)).unwrap_unchecked() }
    }
}

impl<'a, P: ArrayElem<'a>> DoubleEndedIterator for RegArrayIter<'a, P> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: calling `next_back_unchecked` is safe since we check len() first.
        unsafe {
            if self.len() == 0 {
                None
            } else {
                Some(self.next_back_unchecked())
            }
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.end = self.start;
            None
        } else {
            // SAFETY: We are in bounds.
            unsafe {
                self.pre_dec_end(n);
                Some(self.next_back_unchecked())
            }
        }
    }
}

impl<'a, P: ArrayElem<'a>> FusedIterator for RegArrayIter<'a, P> {}
