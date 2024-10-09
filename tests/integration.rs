use reg_map::RegMap;

#[repr(C)]
#[derive(RegMap, Default)]
pub struct Simple {
    field1: u64,
    field2: u64,
}

#[repr(C)]
#[derive(RegMap)]
struct Nested {
    field1: u64,
    field2: NestedInner,
}
#[repr(C)]
#[derive(RegMap)]
struct NestedInner {
    inner1: u64,
}

#[repr(C)]
#[derive(RegMap, Default)]
struct Array {
    field1: u64,
    field2: [u64; 32],
}

#[repr(C)]
#[derive(RegMap)]
struct MixedU {
    one: u8,
    two: u16,
    four: u32,
    eight: u64,
    sixteen: u128,
}
#[repr(C)]
#[derive(RegMap)]
struct MixedI {
    one: i8,
    two: i16,
    four: i32,
    eight: i64,
    sixteen: i128,
}

#[repr(C)]
#[derive(RegMap)]
struct MixedReverse {
    sixteen: u128,
    eight: u64,
    four: u32,
    two: u16,
    one: u8,
}

const LENGTH: usize = 2;
#[repr(C)]
#[derive(RegMap, Default)]
struct ComplexArray {
    field1: [SubArray; LENGTH * 2],
}
#[repr(C)]
#[derive(RegMap, Default)]
struct SubArray {
    field1: u64,
}

#[repr(C)]
#[derive(RegMap, Default)]
struct CAOuter {
    outer: [CAInner; 7],
}
#[repr(C)]
#[derive(RegMap, Default)]
struct CAInner {
    inner: [u64; 11],
}

#[repr(C)]
#[derive(RegMap, Default)]
struct Array4d {
    data: [[[[u64; 2]; 3]; 5]; 7],
}

#[repr(C)]
#[derive(RegMap, Default)]
struct Array4dComplex {
    data: [[[[Simple; 2]; 3]; 5]; 7],
}

#[test]
fn simple() {
    let mut regs = Simple {
        field1: 23,
        field2: 45,
    };
    let ptr = SimplePtr::from_mut(&mut regs);

    assert_eq!(ptr.field1().read(), 23);
    assert_eq!(ptr.field2().read(), 45);

    ptr.field1().write(32);
    ptr.field2().write(54);

    assert_eq!(ptr.field1().read(), 32);
    assert_eq!(ptr.field2().read(), 54);

    assert_eq!(regs.field1, 32);
    assert_eq!(regs.field2, 54);
}

#[test]
fn nested() {
    let mut regs = Nested {
        field1: 1,
        field2: NestedInner { inner1: 2 },
    };
    let ptr = NestedPtr::from_mut(&mut regs);

    assert_eq!(ptr.field1().read(), 1);
    assert_eq!(ptr.field2().inner1().read(), 2);

    ptr.field1().write(10);
    ptr.field2().inner1().write(20);

    assert_eq!(ptr.field1().read(), 10);
    assert_eq!(ptr.field2().inner1().read(), 20);

    assert_eq!(regs.field1, 10);
    assert_eq!(regs.field2.inner1, 20);
}

#[test]
fn array_idx() {
    let mut regs = Array::default();
    let ptr = ArrayPtr::from_mut(&mut regs);

    assert_eq!(ptr.field2().len(), 32);

    ptr.field1().write(42);
    for i in 0..ptr.field2().len() {
        ptr.field2().idx(i).write(i as u64 * 2 + 1);
    }

    assert_eq!(ptr.field1().read(), 42);
    for i in 0..ptr.field2().len() {
        assert_eq!(ptr.field2().idx(i).read(), i as u64 * 2 + 1);
    }

    assert_eq!(regs.field1, 42);
    for (i, v) in regs.field2.into_iter().enumerate() {
        assert_eq!(v, i as u64 * 2 + 1);
    }
}

#[test]
fn array_iter() {
    let mut regs = Array::default();
    let ptr = ArrayPtr::from_mut(&mut regs);

    assert_eq!(ptr.field2().len(), 32);

    ptr.field1().write(42);
    for (i, elem) in ptr.field2().iter().enumerate() {
        elem.write(i as u64 * 2 + 1);
    }

    assert_eq!(ptr.field1().read(), 42);
    for (i, elem) in ptr.field2().iter().enumerate() {
        assert_eq!(elem.read(), i as u64 * 2 + 1);
    }

    assert_eq!(regs.field1, 42);
    for (i, v) in regs.field2.into_iter().enumerate() {
        assert_eq!(v, i as u64 * 2 + 1);
    }
}

#[test]
fn mixed() {
    let mut regs_u = MixedU {
        one: 1,
        two: 2,
        four: 4,
        eight: 8,
        sixteen: 16,
    };
    let mut regs_i = MixedI {
        one: -1,
        two: -2,
        four: -4,
        eight: -8,
        sixteen: -16,
    };
    let ptr_u = MixedUPtr::from_mut(&mut regs_u);
    let ptr_i = MixedIPtr::from_mut(&mut regs_i);

    assert_eq!(ptr_u.one().read(), 1u8);
    assert_eq!(ptr_u.two().read(), 2u16);
    assert_eq!(ptr_u.four().read(), 4u32);
    assert_eq!(ptr_u.eight().read(), 8u64);
    assert_eq!(ptr_u.sixteen().read(), 16u128);

    assert_eq!(ptr_i.one().read(), -1i8);
    assert_eq!(ptr_i.two().read(), -2i16);
    assert_eq!(ptr_i.four().read(), -4i32);
    assert_eq!(ptr_i.eight().read(), -8i64);
    assert_eq!(ptr_i.sixteen().read(), -16i128);

    ptr_u.one().write(10u8);
    ptr_u.two().write(20u16);
    ptr_u.four().write(40u32);
    ptr_u.eight().write(80u64);
    ptr_u.sixteen().write(160u128);

    ptr_i.one().write(-10i8);
    ptr_i.two().write(-20i16);
    ptr_i.four().write(-40i32);
    ptr_i.eight().write(-80i64);
    ptr_i.sixteen().write(-160i128);

    assert_eq!(ptr_u.one().read(), 10u8);
    assert_eq!(ptr_u.two().read(), 20u16);
    assert_eq!(ptr_u.four().read(), 40u32);
    assert_eq!(ptr_u.eight().read(), 80u64);
    assert_eq!(ptr_u.sixteen().read(), 160u128);

    assert_eq!(ptr_i.one().read(), -10i8);
    assert_eq!(ptr_i.two().read(), -20i16);
    assert_eq!(ptr_i.four().read(), -40i32);
    assert_eq!(ptr_i.eight().read(), -80i64);
    assert_eq!(ptr_i.sixteen().read(), -160i128);

    assert_eq!(regs_u.one, 10u8);
    assert_eq!(regs_u.two, 20u16);
    assert_eq!(regs_u.four, 40u32);
    assert_eq!(regs_u.eight, 80u64);
    assert_eq!(regs_u.sixteen, 160u128);

    assert_eq!(regs_i.one, -10i8);
    assert_eq!(regs_i.two, -20i16);
    assert_eq!(regs_i.four, -40i32);
    assert_eq!(regs_i.eight, -80i64);
    assert_eq!(regs_i.sixteen, -160i128);
}

#[test]
fn mixed_reverse() {
    let mut regs = MixedReverse {
        one: 1,
        two: 2,
        four: 4,
        eight: 8,
        sixteen: 16,
    };
    let ptr = MixedReversePtr::from_mut(&mut regs);

    assert_eq!(ptr.one().read(), 1u8);
    assert_eq!(ptr.two().read(), 2u16);
    assert_eq!(ptr.four().read(), 4u32);
    assert_eq!(ptr.eight().read(), 8u64);
    assert_eq!(ptr.sixteen().read(), 16u128);

    ptr.one().write(10u8);
    ptr.two().write(20u16);
    ptr.four().write(40u32);
    ptr.eight().write(80u64);
    ptr.sixteen().write(160u128);

    assert_eq!(ptr.one().read(), 10u8);
    assert_eq!(ptr.two().read(), 20u16);
    assert_eq!(ptr.four().read(), 40u32);
    assert_eq!(ptr.eight().read(), 80u64);
    assert_eq!(ptr.sixteen().read(), 160u128);

    assert_eq!(regs.one, 10u8);
    assert_eq!(regs.two, 20u16);
    assert_eq!(regs.four, 40u32);
    assert_eq!(regs.eight, 80u64);
    assert_eq!(regs.sixteen, 160u128);
}

#[test]
fn leak() {
    let ptr = {
        let regs = Box::new(Simple::default());
        let ptr = SimplePtr::from_mut(Box::leak(regs));
        ptr.field1().write(2);
        ptr
    };
    assert_eq!(ptr.field1().read(), 2);

    let regs = unsafe {
        let raw = ptr.as_ptr();
        #[allow(clippy::drop_non_drop)]
        drop(ptr);
        Box::from_raw(raw)
    };
    drop(regs);
}

#[test]
fn complex_array_idx() {
    let mut regs = ComplexArray::default();
    let ptr = ComplexArrayPtr::from_mut(&mut regs);

    for i in 0..ptr.field1().len() {
        ptr.field1().idx(i).field1().write(1 << i);
    }
    for i in 0..ptr.field1().len() {
        assert_eq!(ptr.field1().idx(i).field1().read(), 1 << i);
    }
    for (i, v) in regs.field1.iter().enumerate() {
        assert_eq!(v.field1, 1 << i);
    }
}
#[test]
fn complex_array_iter() {
    let mut regs = ComplexArray::default();
    let ptr = ComplexArrayPtr::from_mut(&mut regs);

    for (i, elem) in ptr.field1().iter().enumerate() {
        elem.field1().write(1 << i);
    }
    for (i, elem) in ptr.field1().iter().enumerate() {
        assert_eq!(elem.field1().read(), 1 << i);
    }
    for (i, v) in regs.field1.iter().enumerate() {
        assert_eq!(v.field1, 1 << i);
    }
}
#[test]
fn complex_array_slice() {
    let mut regs = ComplexArray::default();
    let ptr = ComplexArrayPtr::from_mut(&mut regs);

    for (i, elem) in ptr.field1().iter_slice(0, 4).enumerate() {
        elem.field1().write(1 << i);
    }
    for (i, elem) in ptr.field1().iter_slice(0, 4).enumerate() {
        assert_eq!(elem.field1().read(), 1 << i);
    }
    for (i, v) in regs.field1.iter().enumerate() {
        assert_eq!(v.field1, 1 << i);
    }
}

#[test]
fn nested_complex_array_idx() {
    let mut regs = CAOuter::default();
    let ptr = CAOuterPtr::from_mut(&mut regs);

    let outer = ptr.outer();
    for i in 0..outer.len() {
        let inner = outer.idx(i).inner();
        for j in 0..inner.len() {
            inner.idx(j).write(((i as u64) << 32) + j as u64);
        }
    }
    let outer = ptr.outer();
    for i in 0..outer.len() {
        let inner = outer.idx(i).inner();
        for j in 0..inner.len() {
            assert_eq!(inner.idx(j).read(), ((i as u64) << 32) + j as u64);
        }
    }
    for (i, outer) in regs.outer.iter().enumerate() {
        for (j, &inner) in outer.inner.iter().enumerate() {
            assert_eq!(inner, ((i as u64) << 32) + j as u64);
        }
    }
}

#[test]
fn nested_complex_array_iter() {
    let mut regs = CAOuter::default();
    let ptr = CAOuterPtr::from_mut(&mut regs);

    for (i, outer) in ptr.outer().iter().enumerate() {
        for (j, inner) in outer.inner().iter().enumerate() {
            inner.write(((i as u64) << 32) + j as u64);
        }
    }
    for (i, outer) in ptr.outer().iter().enumerate() {
        for (j, inner) in outer.inner().iter().enumerate() {
            assert_eq!(inner.read(), ((i as u64) << 32) + j as u64);
        }
    }
    for (i, outer) in regs.outer.iter().enumerate() {
        for (j, &inner) in outer.inner.iter().enumerate() {
            assert_eq!(inner, ((i as u64) << 32) + j as u64);
        }
    }
}

#[test]
fn nested_complex_array_slice() {
    let mut regs = CAOuter::default();
    let ptr = CAOuterPtr::from_mut(&mut regs);

    for (i, outer) in ptr.outer().iter_slice(1, 6).enumerate() {
        for (j, inner) in outer.inner().iter_slice(2, 9).enumerate() {
            inner.write(((i as u64) << 32) + j as u64);
        }
    }
    for (i, outer) in ptr.outer().iter_slice(1, 6).enumerate() {
        for (j, inner) in outer.inner().iter_slice(2, 9).enumerate() {
            assert_eq!(inner.read(), ((i as u64) << 32) + j as u64);
        }
    }
    for (i, outer) in regs.outer[1..6].iter().enumerate() {
        for (j, &inner) in outer.inner[2..9].iter().enumerate() {
            assert_eq!(inner, ((i as u64) << 32) + j as u64);
        }
    }
}

#[test]
fn let_iter_regarr() {
    let mut regs = Array::default();
    let ptr = ArrayPtr::from_mut(&mut regs);

    let mut it = ptr.field2().iter();
    for i in 0..ptr.field2().len() {
        it.next().unwrap().write(i as u64 * 2 + 1);
    }
    assert!(it.next().is_none());

    for (i, elem) in ptr.field2().iter().enumerate() {
        assert_eq!(elem.read(), i as u64 * 2 + 1);
    }
}

#[test]
fn let_iter_ptrarr() {
    let mut regs = ComplexArray::default();
    let ptr = ComplexArrayPtr::from_mut(&mut regs);

    let mut it = ptr.field1().iter();
    for i in 0..ptr.field1().len() {
        it.next().unwrap().field1().write(i as u64 * 2 + 1);
    }
    assert!(it.next().is_none());

    for (i, elem) in ptr.field1().iter().enumerate() {
        assert_eq!(elem.field1().read(), i as u64 * 2 + 1);
    }
}

#[test]
fn array_4d() {
    let mut regs = Array4d::default();
    let dim0 = regs.data.len();
    let dim1 = regs.data[0].len();
    let dim2 = regs.data[0][0].len();
    let dim3 = regs.data[0][0][0].len();

    let ptr = Array4dPtr::from_mut(&mut regs);

    assert_eq!(ptr.data().len(), dim0);
    for (i, a) in ptr.data().iter().enumerate() {
        assert_eq!(a.len(), dim1);
        for (j, b) in a.iter().enumerate() {
            assert_eq!(b.len(), dim2);
            for (k, c) in b.iter().enumerate() {
                assert_eq!(c.len(), dim3);
                for (m, d) in c.iter().enumerate() {
                    assert_eq!(d.read(), 0);
                    d.write((i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }

    for (i, a) in ptr.data().iter().enumerate() {
        for (j, b) in a.iter().enumerate() {
            for (k, c) in b.iter().enumerate() {
                for (m, d) in c.iter().enumerate() {
                    assert_eq!(d.read(), (i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }

    for (i, a) in regs.data.iter().enumerate() {
        for (j, b) in a.iter().enumerate() {
            for (k, c) in b.iter().enumerate() {
                for (m, d) in c.iter().enumerate() {
                    assert_eq!(*d, (i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }
}

#[test]
fn array_4d_complex() {
    let mut regs = Array4dComplex::default();
    let dim0 = regs.data.len();
    let dim1 = regs.data[0].len();
    let dim2 = regs.data[0][0].len();
    let dim3 = regs.data[0][0][0].len();

    let ptr = Array4dComplexPtr::from_mut(&mut regs);

    assert_eq!(ptr.data().len(), dim0);
    for (i, a) in ptr.data().iter().enumerate() {
        assert_eq!(a.len(), dim1);
        for (j, b) in a.iter().enumerate() {
            assert_eq!(b.len(), dim2);
            for (k, c) in b.iter().enumerate() {
                assert_eq!(c.len(), dim3);
                for (m, d) in c.iter().enumerate() {
                    let field1 = d.field1();
                    let field2 = d.field2();
                    assert_eq!(field1.read(), 0);
                    assert_eq!(field2.read(), 0);
                    field1.write((i * dim1 + j * dim2 + k * dim3 + m) as u64);
                    field2.write(!(i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }

    for (i, a) in ptr.data().iter().enumerate() {
        for (j, b) in a.iter().enumerate() {
            for (k, c) in b.iter().enumerate() {
                for (m, d) in c.iter().enumerate() {
                    let field1 = d.field1();
                    let field2 = d.field2();
                    assert_eq!(field1.read(), (i * dim1 + j * dim2 + k * dim3 + m) as u64);
                    assert_eq!(field2.read(), !(i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }

    for (i, a) in regs.data.iter().enumerate() {
        for (j, b) in a.iter().enumerate() {
            for (k, c) in b.iter().enumerate() {
                for (m, d) in c.iter().enumerate() {
                    assert_eq!(d.field1, (i * dim1 + j * dim2 + k * dim3 + m) as u64);
                    assert_eq!(d.field2, !(i * dim1 + j * dim2 + k * dim3 + m) as u64);
                }
            }
        }
    }
}
