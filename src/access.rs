//! Helper types and traits to define read/write permissions on registers.

use core::fmt::Debug;
use core::hash::Hash;

/// A zero-sized type indicating that a register provides only read access.
///
/// Implements the [`Readable`] trait.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadOnly {}

/// A zero-sized type indicating that a register provides only write access.
///
/// Implements the [`Writable`] trait.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteOnly {}

/// A zero-sized type indicating that a register provides both read and write access.
///
/// Implements the [`Readable`] and [`Writable`] traits.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadWrite {}

/// Marker trait required by traits [`Readable`] and [`Writable`];
///
/// ⚠️ This trait is sealed and cannot be implemented for types outside of this crate.
pub trait Access:
    Debug + Default + Copy + Eq + Ord + Hash + Sized + Send + Sync + 'static + private::Sealed
{
}

/// Marker trait for readable registers implemented by types [`ReadOnly`] and [`ReadWrite`].
///
/// ⚠️ This trait is sealed and cannot be implemented for types outside of this crate.
#[diagnostic::on_unimplemented(
    message = "cannot read from a write-only register",
    label = "method cannot be called on write-only registers",
    note = "the register is write only because it was annotated with the attribute
  `#[reg(WO)]` in the register-map definition"
)]
pub trait Readable: Access {}

/// Marker trait for writable registers implemented by types [`WriteOnly`] and [`ReadWrite`].
///
/// ⚠️ This trait is sealed and cannot be implemented for types outside of this crate.
#[diagnostic::on_unimplemented(
    message = "cannot write to a read-only register",
    label = "method cannot be called on read-only registers",
    note = "the register is read only because it was annotated with the attribute
  `#[reg(RO)]` in the register-map definition"
)]
pub trait Writable: Access {}

impl Access for ReadOnly {}
impl Access for WriteOnly {}
impl Access for ReadWrite {}
impl Readable for ReadOnly {}
impl Readable for ReadWrite {}
impl Writable for WriteOnly {}
impl Writable for ReadWrite {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::ReadOnly {}
    impl Sealed for super::WriteOnly {}
    impl Sealed for super::ReadWrite {}
}
