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

Based heavily off [inline-python](https://github.com/fusion-engineering/inline-python) and the [related blog posts](https://blog.m-ou.se/tags/inline-python/).
Thanks @m-ou-se!

### TODO

- [x] Correct white spacing
- [ ] Translate error positions reported by naga to correct locations in Rust source.
