use std::env;

extern crate version_check;

fn main() {
    match version_check::is_min_version("1.27.0") {
        Some((true, _version)) => enable_simd(),
        Some((false, _version)) => (),
        _ => panic!("Unexpected cargo version"),
    }
    println!("Done build.rs")
}

fn enable_simd() {
    if env::var_os("CARGO_FEATURE_STD").is_none() {
        return;
    }

    match env::var_os("CARGO_CFG_TARGET_ARCH") {
        Some(var) => match var.to_str() {
            Some("x86") | Some("x86_64") => println!("cargo:rustc-cfg=thhp_enable_sse42"),
            Some(_) => (),
            None => println!(
                "cargo:warning=CARGO_CFG_TARGET_FEATURE is containind invalid utf-8 characters"
            ),
        },
        None => (),
    }
}
