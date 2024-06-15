
# local-or-heap

This crate provides a type with a pre-specified size, regardless of the size of the `T` type it contains, using heap allocation if necessary.

The purpose of this crate is allow the size of a generic to argument to dictate where it is stored.  This is useful when you want to pack the layout of structures, e.g. for optimizing memory access, but the structure contains a generic type parameter with a size that is unknown to you.

If `size_of::<T>() <= size_of::<SizeT>()` performance should be as close to using the raw type as possible.  If `size_of::<T>() > size_of::<SizeT>()`, performance should be as close to a box as possible.

This crate serves a similar function to [smallbox](https://crates.io/crates/smallbox) but it has slightly lower runtime overhead (memory and instructions) in exchange for not being able to handle dynamically sized types.

## Basic Usage
```rust
    use local_or_heap::LocalOrHeap;

    let int_obj = LocalOrHeap::<usize>::new(42);
    assert_eq!(LocalOrHeap::<usize>::is_heap(), false);
    assert_eq!(&*int_obj, &42);

    let buf_obj = LocalOrHeap::<[usize; 8]>::new([0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(LocalOrHeap::<[usize; 8]>::is_heap(), true);
    assert_eq!(buf_obj.as_ref(), &[0, 1, 2, 3, 4, 5, 6, 7]);
```

## With a Custom Size
```rust
    use local_or_heap::LocalOrHeap;
    type LoH = LocalOrHeap::<[usize; 8], [u8; 1024]>;

    let buf_obj = LoH::new([0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(LoH::is_heap(), false);
    assert_eq!(&*buf_obj, &[0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(core::mem::size_of::<LoH>(), 1024);
```