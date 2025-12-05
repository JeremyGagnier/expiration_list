# ExpirationList

A datastructure for items that expire.

`ExpirationList` is more performant than a `HashMap` for items that are likely to be removed over time and require a stable ID which can be a `usize`. It does not automatically remove items or track expiry. It should not be used when the expiration is fixed, in this case there are other more efficient datastructures such as priority queues.

## Examples

Add and remove:
```rust
use expiration_list::ExpirationList;

let mut list = ExpirationList::new();
let value: i32 = 1234;
let id = list.add(value);
assert_eq!(list.get(id), Some(&value));

let removed_value: i32 = list.remove(id).ok_or(0)?;
assert_eq!(removed_value, value);
assert_eq!(list.get(id), None);
```

Adding and removing many items:
```rust
use expiration_list::ExpirationList;

let mut list = ExpirationList::new();
for idx in 0..10_000 {
	list.add(idx);
}
assert!(list.capacity() >= 10_000);

for idx in 0..(10_000 - 10) {
	list.remove(idx);
}

assert_eq!(list.len(), 10);
// The capacity of the inner structures are reduced
assert_eq!(list.capacity(), 40);
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
