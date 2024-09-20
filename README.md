# `reg-map`

[![Crates.io Version](https://img.shields.io/crates/v/reg-map)](https://crates.io/crates/reg-map)
[![docs.rs](https://img.shields.io/docsrs/reg-map)](https://docs.rs/reg-map)

Derive volatile accesses to a register map and memory-mapped IO.

## Documentation

API documentation in available at [docs.rs/reg-map](https://docs.rs/reg-map).

## Usage

Here's a quick-and-dirty example, see the [API documentation](https://docs.rs/reg-map) for more use
cases such as iteration and a more in-depth description of the functionality.

```rust
use reg_map::RegMap;

// define struct Registers with the register map
// and derive the pointer RegistersPtr using the RegMap macro
#[repr(C)]
#[derive(RegMap, Default)]
struct Registers {
    field1: u64,
    field2: u32,
    #[reg(RO)]
    read_only_field: i8,
    #[reg(WO)]
    write_only_field: u128,
    #[reg(RW)]
    read_write_is_default: i16,
}

// initialize the base struct
// and obtain a pointer to the registers
let mut regs = Registers::default();
let ptr = RegistersPtr::from_mut(&mut regs);

// when dealing with e.g. memory-mapped IO (MMIO),
// you'd probably just get a pointer to the data from a known base address
// let ptr = unsafe { RegistersPtr::from_ptr(0xAA55_000 as *mut _) };

// all write() operations are volatile
ptr.field1().write(10);
ptr.field2().write(32);
ptr.write_only_field().write(76);
ptr.read_write_is_default().write(98);

// all read() operations are volatile
assert_eq!(ptr.field1().read(), 10);
assert_eq!(ptr.field2().read(), 32);
assert_eq!(ptr.read_only_field().read(), 0);
assert_eq!(ptr.read_write_is_default().read(), 98);
```

## License

Licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  (see [LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT)
  (see [LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
