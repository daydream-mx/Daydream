#![recursion_limit = "1024"]
//====== Running the worker ======///

// We need to import the Threaded trait to register the worker
use wasi_worker::{ServiceOptions, ServiceWorker};
use wasi_worker_yew::WASIAgent;

/// This gets called by the worker.js entrypoint
/// We need to wrap it in wasm_bindgen so the worker knows the spin the the yew worker instance
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    // Spawning a yew component without StartApp requires initializing
    // yew::initialize();
    let opt = ServiceOptions::default().with_cleanup();
    ServiceWorker::initialize(opt).expect("ServiceWorker::initialize");
    let agent = WASIAgent::<daydream::matrix::MatrixAgent>::new();

    ServiceWorker::set_message_handler(Box::new(agent));
}
