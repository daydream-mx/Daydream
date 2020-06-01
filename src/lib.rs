#![recursion_limit = "512"]
extern crate console_error_panic_hook;
extern crate wee_alloc;

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;

mod app;
mod constants;
mod errors;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();
    Ok(())
}
