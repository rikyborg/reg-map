//! Types that can be placed into a [`Reg`](crate::reg::Reg).

use core::fmt::Debug;
use core::hash::Hash;

/// Types that can be placed into a [`Reg`](crate::reg::Reg).
///
/// This trait is implemented on all primitive integer types *except* the pointer-sized types
/// `usize` and `isize`.
///
/// ⚠️ This trait is sealed and cannot be implemented for types outside of this crate.
pub trait Integer:
    Debug + Default + Copy + Eq + Ord + Hash + Sized + Send + Sync + 'static + private::Sealed
{
}

impl Integer for u8 {}
impl Integer for u16 {}
impl Integer for u32 {}
impl Integer for u64 {}
impl Integer for u128 {}
impl Integer for i8 {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}
impl Integer for i128 {}

mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for u128 {}
    impl Sealed for i8 {}
    impl Sealed for i16 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for i128 {}
}
