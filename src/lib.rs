//! Derive volatile accesses to a register map and memory-mapped IO.
//!
//! The main entry point of this crate is the derive macro [`RegMap`], that generates a new pointer
//! type to a defined register map.
//!
//! **Table of contents**
//! - [Basic usage](#basic-usage)
//! - [Register types](#register-types)
//!   - [Basic registers](#basic-registers)
//!   - [Nested register maps](#nested-register-maps)
//!   - [Arrays of registers](#arrays-of-registers)
//!     - [Iterators](#iterators)
//! - [Access permissions](#access-permissions)
//! - [Type layout and representation](#type-layout-and-representation)
//! - [Thread safety](#thread-safety)
//! - [Crate features](#crate-features)
//! - [Principle of operation](#principle-of-operation)
//!   - [Sample generated code](#sample-generated-code)
//! - [Comparison with other crates](#comparison-with-other-crates)
//!   - [volatile](#volatile)
//!   - [volatile-register](#volatile-register)
//! - [Further reading](#further-reading)
//!
//! # Basic usage
//!
//! ```rust
//! # mod yoo {
//! # use reg_map::RegMap;
//! // define struct Registers with the register map
//! // and derive the pointer RegistersPtr using the RegMap macro
//! #[repr(C)]
//! #[derive(RegMap, Default)]
//! # pub
//! struct Registers {
//!     field1: u64,
//!     field2: u32,
//!     #[reg(RO)]
//!     read_only_field: i8,
//!     #[reg(WO)]
//!     write_only_field: u128,
//!     #[reg(RW)]
//!     read_write_is_default: i16,
//! }
//! # } // mod yoo
//! # use yoo::{Registers, RegistersPtr};
//!
//! // initialize the base struct
//! // and obtain a pointer to the registers
//! let mut regs = Registers::default();
//! let ptr = RegistersPtr::from_mut(&mut regs);
//!
//! // when dealing with e.g. memory-mapped IO (MMIO),
//! // you'd probably just get a pointer to the data from a known base address
//! // let ptr = unsafe { RegistersPtr::from_ptr(0xAA55_000 as *mut _) };
//!
//! // all write() operations are volatile
//! ptr.field1().write(10);
//! ptr.field2().write(32);
//! ptr.write_only_field().write(76);
//! ptr.read_write_is_default().write(98);
//!
//! // all read() operations are volatile
//! assert_eq!(ptr.field1().read(), 10);
//! assert_eq!(ptr.field2().read(), 32);
//! assert_eq!(ptr.read_only_field().read(), 0);
//! assert_eq!(ptr.read_write_is_default().read(), 98);
//! ```
//!
//! Read/write permissions are checked at compile time. The following code does not compile:
//! ```compile_fail,E0277
//! # mod yoo {
//! # use reg_map::RegMap;
//! # #[repr(C)]
//! # #[derive(RegMap, Default)]
//! # pub struct Registers {
//! #     field1: u64,
//! #     field2: u32,
//! #     #[reg(RO)]
//! #     read_only_field: i8,
//! #     #[reg(WO)]
//! #     write_only_field: u128,
//! #     #[reg(RW)]
//! #     read_write_is_default: i16,
//! # }
//! # } // mod yoo
//! # use yoo::{Registers, RegistersPtr};
//! # let mut regs = Registers::default();
//! # let ptr = RegistersPtr::from_mut(&mut regs);
//! ptr.read_only_field().write(54); // error[E0277]: cannot write to a read-only register
//! ptr.write_only_field().read();   // error[E0277]: cannot read from a write-only register
//! ```
//!
//! # Register types
//!
//! ## Basic registers
//!
//! These primitive integer types are supported as basic register types:
//! - unsigned [`u8`], [`u16`], [`u32`], [`u64`] and [`u128`],
//! - signed [`i8`], [`i16`], [`i32`], [`i64`] and [`i128`].
//!
//! The pointer-sized integer types [`usize`] and [`isize`] are *not* supported.
//!
//! For a register map containing a basic register:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! #[derive(RegMap, Default)]
//! #[repr(C)]
//! struct Basic {
//!     field: u64,
//! }
//! # } // mod yoo
//! ```
//! The [`RegMap`] derive macro will generate the following abridged code:
//! ```ignore
//! struct BasicPtr<'a> { ... };
//! impl<'a> BasicPtr<'a> {
//!     fn field(&self) -> Reg<'a, u64, ReadWrite> { ... }
//! }
//! ```
//! where the read/write operations on the register are performed through the [`Reg`] type, and the
//! access permissions default to both read and write.
//!
//! ## Nested register maps
//! Register-map definitions can be nested arbitrarily:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! # #[derive(RegMap, Default)]
//! # #[repr(C)]
//! # struct Basic {
//! #     field: u64,
//! # }
//! #[derive(RegMap)]
//! #[repr(C)]
//! struct Outer {
//!     outer: u64,
//!     inner: Basic,
//! }
//! # } // mod yoo
//! ```
//! will generate pointer types with the following abridged code:
//! ```ignore
//! struct OuterPtr<'a> { ... };
//! impl<'a> OuterPtr<'a> {
//!     fn outer(&self) -> Reg<'a, u64, ReadWrite> { ... }
//!     fn inner(&self) -> BasicPtr<'a> { ... }
//! }
//! ```
//! where `Basic` and `BasicPtr` are shown in the previous section.
//!
//! ## Arrays of registers
//! Fixed-size arrays of registers are also supported, with both basic and nested registers.
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! # #[derive(RegMap, Default)]
//! # #[repr(C)]
//! # struct Basic {
//! #     field: u64,
//! # }
//! #[derive(RegMap, Default)]
//! #[repr(C)]
//! struct Many {
//!     basic: [u64; 32],
//!     nested: [Basic; 16],
//! }
//! # } // mod yoo
//! ```
//! generates the following abridged code:
//! ```ignore
//! struct ManyPtr<'a> { ... };
//! impl<'a> ManyPtr<'a> {
//!     fn basic(&self) -> RegArray<'a, Reg<'a, u64, ReadWrite>, 32> { ... }
//!     fn nested(&self) -> RegArray<'a, BasicPtr<'a>, 16> { ... }
//! }
//! ```
//! where the access to the arrays of registers are provided by the [`RegArray`] type.
//!
//! Multidimensional arrays are also supported:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! # #[derive(RegMap)]
//! # #[repr(C)]
//! # struct Basic {
//! #     field: u64,
//! # }
//! #[derive(RegMap)]
//! #[repr(C)]
//! struct MultiD {
//!     basic: [[[[u64; 2]; 3]; 5]; 7],
//!     nested: [[[[Basic; 7]; 5]; 3]; 2],
//! }
//! # } // mod yoo
//! ```
//!
//! ### Iterators
//!
//! It is possible to iterate through arrays using the methods [`RegArray::iter`] and
//! [`RegArray::iter_slice`]:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! # #[derive(RegMap, Default)]
//! # #[repr(C)]
//! # pub struct Basic {
//! #     pub field: u64,
//! # }
//! # #[derive(RegMap, Default)]
//! # #[repr(C)]
//! # pub struct Many {
//! #     pub basic: [u64; 32],
//! #     pub nested: [Basic; 16],
//! # }
//! # } // mod yoo
//! # use yoo::{Many, ManyPtr};
//! let mut reg = Many::default();
//! let ptr = ManyPtr::from_mut(&mut reg);
//!
//! for (i, basic) in ptr.basic().iter().enumerate() {
//!     basic.write(i as u64);
//! }
//! for (i, basic) in ptr.basic().iter().enumerate() {
//!     assert_eq!(basic.read(), i as u64);
//! }
//!
//! for (j, nested) in ptr.nested().iter_slice(2, 7).rev().enumerate() {
//!     nested.field().write(j as u64);
//! }
//! for (j, nested) in ptr.nested().iter().enumerate() {
//!     let expected = if (2..7).contains(&j) {
//!         6 - j
//!     } else {
//!         0
//!     };
//!     assert_eq!(nested.field().read(), expected as u64);
//! }
//! ```
//!
//! # Access permissions
//! Access permissions for each register can be specified with the `#[reg()]` attribute, and
//! default to read-write if not specified:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! #[repr(C)]
//! #[derive(RegMap)]
//! struct Permissions {
//!     #[reg(RO)] read_only_register: u64,
//!     #[reg(WO)] write_only_register: u64,
//!     #[reg(RW)] read_write_register: u64,
//!     another_read_only_register: u64,
//! }
//! # } // mod yoo
//! ```
//! Access permission are implemented through the zero-sized structs:
//! - [`ReadOnly`](access::ReadOnly) for read-only registers (`#[reg(RO)]` attribute);
//! - [`WriteOnly`](access::WriteOnly) for write-only registers (`#[reg(WO)]` attribute);
//! - [`ReadWrite`](access::ReadWrite) for read-write registers (`#[reg(RW)]` attribute, or no attribute).
//!
//! Access permission are checked at compile time, as the zero-sized structs above are passed as
//! type parameters to the generic types [`Reg`] and [`RegArray`] upon definition of the derived
//! pointer types. Specifically, the [`write`](Reg::write) is just not defined for a read-only
//! register, and so on.
//!
//! # Type layout and representation
//! The derive macro [`RegMap`] requires the register-map `struct` to have the `C` representation
//! using the `#[repr(C)]` attribute. Higher alignment requirements can be specified with the
//! `#[repr(C, align(x))]` attribute. Other representations are not supported and generate a
//! compile-time error.
//!
//! Example:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! #[repr(C)]
//! #[derive(RegMap)]
//! struct Base {
//!     foo: u32,
//!     baz: u32,
//!     aligned: Data,
//! }
//! #[repr(C, align(4096))]
//! #[derive(RegMap)]
//! struct Data {
//!     data: [u64; 512],
//! }
//! # } // mod yoo
//! ```
//!
//! In summary:
//! - `#[repr(C)]`: The `C` representation is *required*.
//! - Default/`Rust` representation is *not* supported.
//! - `#[repr(transparent)]`: The `transparent` representation is *not* supported.
//! - `#[repr(align(x))]`: *Raising* the alignment of the register map is supported, in combination
//!   with the `C` representation.
//! - `#[repr(packed)]`: *Lowering* the alignment of the register map is *not* supported.
//!   This is because unaligned reads and writes are not (currently) supported.
//!
//! # Thread safety
//!
//! All reads and writes performed through the pointers derived by [`RegMap`] are volatile. However
//! in Rust, *"just like in C, whether an operation is volatile has no bearing whatsoever on
//! questions involving concurrent access from multiple threads. Volatile accesses behave exactly
//! like non-atomic accesses in that regard."* See safety docs for
//! [`read_volatile`](core::ptr::read_volatile#safety) and
//! [`write_volatile`](core::ptr::write_volatile#safety).
//!
//! There is currently no way in Rust to define memory accesses as both volatile and atomic.
//! Therefore, the pointers derived by [`RegMap`] are generally not thread safe and thus implement
//! neither [`Send`] not [`Sync`].
//!
//! That said, on some platforms and for some use cases, volatile access and relaxed atomic
//! accesses are the same. If you know that is the case, you can `unsafe`ly implement `Send` and
//! `Sync` yourself:
//!
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! #[repr(C)]
//! #[derive(RegMap)]
//! # pub
//! struct IPromiseThisIsThreadSafe {
//!     data: u64,
//! }
//! # } // mod yoo
//! # use yoo::IPromiseThisIsThreadSafePtr;
//! // Safety: I did my homework and this is sound on
//! // my platform and for my use case. I promise!
//! unsafe impl Send for IPromiseThisIsThreadSafePtr<'static> {}
//! unsafe impl Sync for IPromiseThisIsThreadSafePtr<'static> {}
//! ```
//!
//! If something goes wrong, that's on you! See also
//! [URLO: Volatile + relaxed atomic load/store](https://users.rust-lang.org/t/volatile-relaxed-atomic-load-store/92792).
//!
//! # Crate features
//!
//! By default, no features are enabled. These features exist:
//!
//! - **std** -
//!   When enabled, this will cause `reg-map` to use the standard library. Currently, this feature
//!   is only used as a dependency of other features.
//!
//! - **debug-trace** -
//!   When enabled, all register reads and writes print a debug trace to standard error. Depends on
//!   feature `std`. For example, the code
//!   ```ignore
//!   ptr.field1().write(0);
//!   ptr.field1().read();
//!   ptr.field2().write(0xa5a5);
//!   ptr.field2().read();
//!   ```
//!   might print something like
//!   ```text
//!   REG-MAP WRITE 0x7ffc30c85c70 0
//!   REG-MAP READ  0x7ffc30c85c70 0
//!   REG-MAP WRITE 0x7ffc30c85c78 42405
//!   REG-MAP READ  0x7ffc30c85c78 42405
//!   ```
//!   Note that this feature only works on targets that support `std`, and that printing to
//!   standard error for every register access might heavily impact performance.
//!
//! # Principle of operation
//!
//! The derive macro [`RegMap`] takes as input the definition of a register map (a `struct`), and
//! generates a custom pointer type that is a wrapper around a raw pointer to the original
//! `struct`. This custom pointer provides methods to perform read / write volatile operations on
//! the fields of the register map.
//!
//! Importantly, no references to the original register map need to ever be created. Instead, the
//! derive macro uses the original `struct` definition to calculate the offsets needed for each
//! memory access. The memory accesses are always performed on raw pointers with volatile
//! semantics.
//!
//! Avoiding creation of references to volatile memory is important to ensure soundness, as
//! discussed e.g. in
//! [rust-lang/unsafe-code-guidelines#33](https://github.com/rust-lang/unsafe-code-guidelines/issues/33)
//! and
//! [rust-lang/unsafe-code-guidelines#411](https://github.com/rust-lang/unsafe-code-guidelines/issues/411).
//!
//! ## Sample generated code
//!
//! Some of the content in this section is considered implementation detail and is not subject to
//! stability guarantees. Nonetheless, it might be useful to have a look at the macro-generated
//! code to get a better understanding of the functionality of this crate.
//!
//! A relatively-simple register-map definition:
//! ```
//! # mod yoo {
//! # use reg_map::RegMap;
//! #[repr(C)]
//! #[derive(RegMap)]
//! struct Test {
//!     scalar_field: u64,
//!     array_field: [u64; 4096],
//! }
//! # } // mod yoo
//! ```
//!
//! generates the following code (comments and docs omitted):
//! ```
//! # mod yoo {
//! #[repr(C)]
//! struct Test {
//!     scalar_field: u64,
//!     array_field: [u64; 4096],
//! }
//!
//! #[allow(non_snake_case)]
//! mod _mod_Test {
//!     use super::*;
//!
//!     pub(super) struct TestPtr<'a> {
//!         ptr: ::core::ptr::NonNull<Test>,
//!         _ref: ::core::marker::PhantomData<&'a Test>,
//!     }
//!
//!     impl<'a> TestPtr<'a> {
//!         #[inline]
//!         const unsafe fn from_nonnull(ptr: ::core::ptr::NonNull<Test>) -> Self {
//!             Self {
//!                 ptr,
//!                 _ref: ::core::marker::PhantomData,
//!             }
//!         }
//!         #[inline]
//!         pub const unsafe fn from_ptr(ptr: *mut Test) -> Self {
//!             Self::from_nonnull(::core::ptr::NonNull::new_unchecked(ptr))
//!         }
//!         #[inline]
//!         pub fn from_mut(reg: &'a mut Test) -> Self {
//!             unsafe { Self::from_ptr(reg) }
//!         }
//!         #[inline]
//!         pub const fn as_ptr(&self) -> *mut Test {
//!             self.ptr.as_ptr()
//!         }
//!         #[inline]
//!         pub fn scalar_field(&self) -> ::reg_map::Reg<'a, u64, ::reg_map::access::ReadWrite> {
//!             unsafe {
//!                 ::reg_map::Reg::__MACRO_ONLY__from_ptr(::core::ptr::addr_of_mut!(
//!                     (*self.as_ptr()).scalar_field
//!                 ))
//!             }
//!         }
//!         #[inline]
//!         pub fn array_field(
//!             &self,
//!         ) -> ::reg_map::RegArray<'a, ::reg_map::Reg<'a, u64, ::reg_map::access::ReadWrite>, 4096>
//!         {
//!             unsafe {
//!                 ::reg_map::RegArray::__MACRO_ONLY__from_ptr(::core::ptr::addr_of_mut!(
//!                     (*self.as_ptr()).array_field
//!                 ))
//!             }
//!         }
//!     }
//!
//!     unsafe impl<'a> ::reg_map::RegMapPtr<'a> for TestPtr<'a> {
//!         type RegMap = Test;
//!         #[inline]
//!         unsafe fn from_nonnull(ptr: ::core::ptr::NonNull<Self::RegMap>) -> Self {
//!             Self::from_nonnull(ptr)
//!         }
//!         #[inline]
//!         unsafe fn from_ptr(ptr: *mut Self::RegMap) -> Self {
//!             Self::from_ptr(ptr)
//!         }
//!         #[inline]
//!         fn from_mut(reg: &'a mut Self::RegMap) -> Self {
//!             Self::from_mut(reg)
//!         }
//!         #[inline]
//!         fn as_ptr(&self) -> *mut Self::RegMap {
//!             self.as_ptr()
//!         }
//!     }
//! }
//!
//! use _mod_Test::TestPtr;
//! # } // mod yoo
//! ```
//!
//! First of all, the derive macro generates a module `_mod_Test` that contains the generated
//! pointer type `TestPtr`. The reason to define the type inside of a module is to enforce that a
//! new pointer is only created through the `pub` associated functions `from_ptr` and `from_mut`.
//! The defined `TestPtr` type is then re-exported out of the module.
//!
//! `TestPtr` itself is just a wrapper around a [`NonNull`](core::ptr::NonNull) pointer, plus a
//! marker field to signal that it is semantically a `&'a Test`.
//!
//! A new `TestPtr` can be safely constructed from a `&mut Test` through `TestPtr::from_mut`, or
//! `unsafe`ly from a `*mut Test` through `TestPtr::from_ptr`. A raw pointer to the underlying data
//! can be obtained from a live `TestPtr` with the method `TestPtr::as_ptr`.
//!
//! The juice of the generated code are the `TestPtr::scalar_field` and `TestPtr::array_field`
//! methods, which use [`addr_of_mut!`](core::ptr::addr_of_mut) to return a [`Reg`] and a
//! [`RegArray`], respectively. These provide read / write volatile access without ever creating a
//! reference to the underlying data.
//!
//! Finally, the generated code implements the [`RegMapPtr`] trait on `TestPtr` so that it can be
//! stored in a [`RegArray`], if needed.
//!
//! # Comparison with other crates
//!
//! ## `volatile`
//! The crate [volatile](https://lib.rs/crates/volatile) uses the same principle of operation as
//! this crate, `reg-map`: a custom pointer type is defined to perform volatile read /write
//! operations to the underlying memory.
//!
//! In fact, `reg-map` is heavily inspired by `volatile`! Differences are mainly ergonomic and in
//! the exposed API surface.
//!
//! ## `volatile-register`
//! The crate [volatile-register](https://lib.rs/crates/volatile-register), based on
//! [vcell](https://lib.rs/crates/vcell), exposes a very clean API by providing a wrapper type that
//! owns the data. In practice, it offers a `VolatileCell` which is *"just like
//! [`Cell`](core::cell::Cell) but with volatile read / write operations"*.
//!
//! Unfortunately, this approach is unsound. See [rust-lang/unsafe-code-guidelines#33: What about: volatile accesses and memory-mapped IO](https://github.com/rust-lang/unsafe-code-guidelines/issues/33)
//! for details. Long story short, every time there is a `&UnsafeCell<T>`, the compiler is allowed
//! to insert spurious reads and writes. That is fine for "normal memory", but with volatile memory
//! and memory-mapped IO reads and writes have side effects so this problematic.
//!
//! This crate, `reg-map`, does not use `UnsafeCell` and never creates references to the volatile
//! memory, avoiding the soundness issue above.
//!
//! For the cases where the `volatile-register` approach happens to work, the assembly generated by
//! the two approaches is identical.
//!
//! # Further reading
//!
//! Some links to relevant forum threads and GitHub issues:
//! - [URLO: How to make an access volatile without std library?](https://users.rust-lang.org/t/how-to-make-an-access-volatile-without-std-library/85533)
//! - [URLO: Volatile + relaxed atomic load/store](https://users.rust-lang.org/t/volatile-relaxed-atomic-load-store/92792)
//! - [URLO: Why are memory mapped registers implemented with interior mutability?](https://users.rust-lang.org/t/why-are-memory-mapped-registers-implemented-with-interior-mutability/116119)
//! - [rust-embedded/volatile-register#10: Usage of references is in conflict with use for MMIO](https://github.com/rust-embedded/volatile-register/issues/10)
//! - [rust-lang/unsafe-code-guidelines#33: What about: volatile accesses and memory-mapped IO](https://github.com/rust-lang/unsafe-code-guidelines/issues/33)
//! - [rust-lang/unsafe-code-guidelines#411: Can we have VolatileCell](https://github.com/rust-lang/unsafe-code-guidelines/issues/411)

#![no_std]

/// Derive macro to generate a pointer to a register map with volatile reads and writes.
///
/// See the [top-level documentation](crate) for usage information and examples.
pub use reg_map_derive::RegMap;

pub mod access;

mod arr;
pub use arr::{ArrayElem, RegArray};

mod bounds;

pub mod integers;

mod iter;

mod reg;
pub use reg::{Reg, RegMapPtr};
