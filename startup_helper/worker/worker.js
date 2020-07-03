import("../../pkg").then(wasm => {
    // Call the library function we exported with wasm-bindgen in lib.rs
    wasm.init_worker();
});
