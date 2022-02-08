# MagicString (it's not really magic)

A string type made up of multiple string slices, that makes no heap allocations.

```rust
let a = "hello ";
let b = "world";
let slices = [a, b];

let string = MagicString::new(&slices);

println!("{string}");
```

Works with `#![no_std]`.
