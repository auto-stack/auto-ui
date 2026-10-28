// The VM executor runs on this binary's OS main thread (winit requirement).
// RUST_MIN_STACK only sizes spawned Rust threads, not the Windows main thread.
// Keep the owning tool package's main-stack policy aligned with the Man CLI.
fn main() {
    println!("cargo:rerun-if-env-changed=AUTO_STACK_SIZE");
    println!("cargo:rerun-if-env-changed=auto_stack_size");
    let stack = std::env::var("AUTO_STACK_SIZE")
        .or_else(|_| std::env::var("auto_stack_size"))
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64 * 1024 * 1024);
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let abi = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os == "windows" {
        match abi.as_str() {
            "msvc" => println!("cargo:rustc-link-arg-bin=auto-ui-tool=/STACK:{stack}"),
            "gnu" => println!("cargo:rustc-link-arg-bin=auto-ui-tool=-Wl,--stack,{stack}"),
            _ => {}
        }
    }
}
