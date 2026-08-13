fn main() {
    use std::env;
    let is_windows = env::var("CARGO_CFG_WINDOWS").is_ok();

    if is_windows {
        println!("cargo:rustc-link-lib=./icon_resources/res");
    }
}
