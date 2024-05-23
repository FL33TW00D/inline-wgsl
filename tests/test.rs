#[cfg(test)]
mod tests {
    use inline_wgsl::wgsl;

    #[test]
    pub fn test_wgsl() {
        let reduce_var = 1024;
        let templated = wgsl! {
            for(var i: u32 = index; i < 'reduce_var; i += BLOCK_SIZE) {
                var val = X[row_start + i];
                X[row_start + i] = exp(val - maximum) / sum;
            }
        };

        println!("{}", templated);
    }
}
