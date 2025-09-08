#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
pub fn init() {
    // placeholder for wasm startup
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init() {}
