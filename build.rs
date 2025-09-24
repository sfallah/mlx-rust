// build.rs
use std::{env, path::PathBuf};

fn main() {
    // --- Configuration knobs (override via env if you like) ---
    // Path to the mlx-c submodule root (contains CMakeLists.txt)
    let mlx_c_src = env::var("MLX_C_SRC").unwrap_or_else(|_| "mlx-c".into());

    let header_path = PathBuf::from("wrapper.h");

    // Extra clang flags for bindgen (space-separated). Example: "-DMYFLAG=1 -I/path/include"
    let extra_clang_args: Vec<String> = env::var("BINDGEN_EXTRA_CLANG_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // Deployment target for macOS builds (can override). Keep in sync with your project.
    let macos_min_ver = env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "14.0".into());

    // Build profile for CMake (Debug/Release/RelWithDebInfo/MinSizeRel). Default: Release
    let cmake_profile = env::var("MLX_C_CMAKE_PROFILE").unwrap_or_else(|_| "Release".into());

    // Let Cargo know when to re-run build script
    println!("cargo:rerun-if-changed={}", header_path.display());
    println!("cargo:rerun-if-env-changed=MLX_C_SRC");
    println!("cargo:rerun-if-env-changed=MLX_C_HEADER");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=MLX_C_CMAKE_PROFILE");

    // --- Build mlx-c with CMake ---
    let mut cfg = cmake::Config::new(&mlx_c_src);
    cfg.profile(&cmake_profile);

    // Ensure we build static by default unless the project forces shared.
    // (If the upstream ignores this, it's harmless.)
    cfg.define("BUILD_SHARED_LIBS", "OFF");

    // On macOS, set a minimum deployment target so we match the MLX expectations.
    if cfg!(target_os = "macos") {
        cfg.define("CMAKE_OSX_DEPLOYMENT_TARGET", &macos_min_ver);
    }

    // Honor user-provided C/C++ flags if set
    if let Ok(cflags) = env::var("CFLAGS") {
        cfg.cflag(cflags);
    }
    if let Ok(cxxflags) = env::var("CXXFLAGS") {
        cfg.cxxflag(cxxflags);
    }

    // Kick off the build; returns the install-ish output dir where libs/headers land.
    let dst = cfg.build();

    // Typically CMake places built libraries under {dst}/lib or {dst}/lib64.
    let lib_dir = {
        let lib = PathBuf::from(&dst).join("lib");
        let lib64 = PathBuf::from(&dst).join("lib64");
        if lib.is_dir() { lib } else { lib64 }
    };

    // Tell cargo where to find the compiled library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Upstream library name is usually 'mlx-c' (libmlx-c.a/dylib).
    // If upstream changes it, override via MLX_C_LIB_NAME env var.
    println!("cargo:rustc-link-lib=static=mlxc");
    // ALSO link the MLX C++ core lib if you used C++ headers/symbols:
    println!("cargo:rustc-link-lib=static=mlx"); // <-- ensure this actually exists

    // macOS frameworks required by MLX stack
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        // C++ stdlib (mlx-c pulls in C++ in parts of the stack)
        println!("cargo:rustc-link-lib=c++");
    }

    // --- Generate Rust bindings with bindgen ---
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));
    let bindings_out = out_dir.join("bindings.rs");

    // Include search roots for headers: prefer the source tree,
    // but also add the CMake install/include path if present.
    let include_src = PathBuf::from(&mlx_c_src);
    let include_build = PathBuf::from(&dst).join("include");

    let mut builder = bindgen::Builder::default()
        .header(header_path.to_string_lossy())
        // Where bindgen/clang should look for headers
        .clang_arg(format!("-I{}", include_src.display()))
        .clang_arg(format!("-I{}", include_build.display()))
        // macOS: make sure clang sees the correct sysroot/deployment target
        .clang_arg(format!("-mmacosx-version-min={}", macos_min_ver))
        // A reasonable allowlist so we only expose the public C API.
        .allowlist_function("mlx_.*")
        .allowlist_type("mlx_.*")
        .allowlist_var("MLX_.*")
        // Keep layout tests off by default to avoid CI differences across SDKs
        .layout_tests(false)
        // Generate `#[repr(C)]` newtypes for enums (often safer with C APIs).
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        });

    for arg in extra_clang_args {
        builder = builder.clang_arg(arg);
    }

    // If the API needs specific defines, you can add them here, e.g.:
    // builder = builder.clang_arg("-DMLX_C_SOME_FLAG=1");

    let bindings = builder
        .generate()
        .expect("bindgen failed to generate bindings for mlx-c");
    bindings
        .write_to_file(&bindings_out)
        .expect("could not write bindings");

    // Let the library code know where to include the generated bindings from
    // (In your lib.rs, do: include!(concat!(env!("OUT_DIR"), "/bindings.rs")); )
    println!(
        "cargo:warning=Generated bindings at {}",
        bindings_out.display()
    );
}
