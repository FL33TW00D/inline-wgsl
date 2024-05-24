#[cfg(test)]
mod tests {
    use inline_wgsl::wgsl;
    use wgpu::naga::front::wgsl::parse_str;

    #[test]
    pub fn test_for() {
        let reduce_var = 1024;
        let templated = wgsl! {
            for(var i: u32 = index; i < 'reduce_var; i += BLOCK_SIZE) {
                var val = X[row_start + i];
                X[row_start + i] = exp(val - maximum) / sum;
            }
        };

        println!("{}", templated);
    }

    #[test]
    pub fn test_binding() {
        let group = 0;
        let binding = 1;
        let ty = "storage";
        let mode = "read_write";
        let accessor = "array<vec4<f32>>";
        let name = "X";

        let result = wgsl! {
            @group('group) @binding('binding) var<'ty, 'mode> 'name: 'accessor;
        };
        println!("{}", result);
        parse_str(&result).unwrap();
    }

    #[test]
    pub fn test_fn() {
        let result = wgsl! {
            fn block_sum() {
                var x = 0;
                x += 1;
            }
        };
        println!("{}", result);
        parse_str(&result).unwrap();
    }
}
