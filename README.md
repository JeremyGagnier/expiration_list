# ExpirationList

A datastructure for items that expire.

`ExpirationList` is more performant than a `HashMap` for items that are likely to be removed over time and require a stable ID which can be a `usize`. It does not automatically remove items or track expiry. It should not be used when the expiration is fixed, in this case there are other more efficient datastructures such as priority queues.

Example usage:
```rust
let mut list = ExpirationList::new();
let value: i32 = 1234;
let id = list.add(value);
assert_eq!(list.get(id), Some(&value));
assert_eq!(list.contains(id), true);

let removed_value: i32 = list.remove(id).ok_or(0)?;
assert_eq!(removed_value, value);
assert_eq!(list.get(id), None);
assert_eq!(list.contains(id), false);
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
