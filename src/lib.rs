#![recursion_limit = "1024"]

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod app;
mod constants;
mod errors;
pub mod matrix;
pub mod utils;
