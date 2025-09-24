#[cfg(test)]
mod tests {
    // tests/example.rs

    use mlx_rust::{mlx_string_new, mlx_version};

    #[test]
    fn example_add_vectors_roundtrip() {
        let version = unsafe {
        let mut version = mlx_string_new();
            mlx_version(&mut version);
        };
    }
}
