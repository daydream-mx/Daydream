#![recursion_limit = "1024"]
//====== Running the worker ======///

// We need to import the Threaded trait to register the worker
use log::*;
use yew::agent::Threaded;

/// This gets called by the worker.js entrypoint
/// We need to wrap it in wasm_bindgen so the worker knows the spin the the yew worker instance
fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    // Spawning a yew component without StartApp requires initializing
    yew::initialize();
    info!("Worker");

    // ... registering the worker
    daydream::app::matrix::MatrixAgent::register();
}
