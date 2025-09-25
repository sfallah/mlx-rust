#[cfg(test)]
mod tests {
    // tests/example.
    use mlx_rust as mlx;

    fn gpu_info() {
        println!("==================================================");
        println!("GPU info:");
        let info = unsafe { mlx::mlx_metal_device_info() };
        println!("architecture: {}", unsafe {
            std::ffi::CStr::from_ptr(info.architecture.as_ptr()).to_string_lossy()
        });
        println!("max_buffer_length: {}", info.max_buffer_length);
        println!(
            "max_recommended_working_set_size: {}",
            info.max_recommended_working_set_size
        );
        println!("memory_size: {}", info.memory_size);
        println!("==================================================");
    }

    fn print_array(msg: &str, arr: mlx::mlx_array) {
        unsafe {
            let mut str_obj = mlx::mlx_string_new();
            mlx::mlx_array_tostring(&mut str_obj, arr);
            let c_str = mlx::mlx_string_data(str_obj);
            let rust_str = std::ffi::CStr::from_ptr(c_str).to_string_lossy();
            println!("{}\n{}", msg, rust_str);
            mlx::mlx_string_free(str_obj);
        }
    }

    #[test]
    fn example_add_vectors_roundtrip() {
        gpu_info();
        unsafe {
            let stream = mlx::mlx_default_gpu_stream_new();

            let data: [f32; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
            let shape: [i32; 2] = [2, 3];
            let mut arr = mlx::mlx_array_new_data(
                data.as_ptr() as *const std::ffi::c_void,
                shape.as_ptr(),
                2,
                mlx::mlx_dtype__MLX_FLOAT32,
            );
            print_array("hello world!", arr);
        }
    }
}
