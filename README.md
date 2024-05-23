## Inline WGSL

Embed raw WGSL code into your Rust!

```rust
let workgroup_size_y = 42;
let shader_snip = wgsl! {
    for(var i = 0; i < 'workgroup_size_y; i+=1) {
        X[i] += 1f;
    }
};
```

Not a particularly robust solution.

[ ] - Correct whitespacing
[ ] - Translate error positions reported by naga to correct locations in Rust source.
