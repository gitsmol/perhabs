#![warn(clippy::all, rust_2021_compatibility)]
#![allow(unused_must_use)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Perhabs - Cognitive Rehab Suite",
        native_options,
        Box::new(|cc| Box::new(app::Perhabs::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let wr = eframe::WebRunner::new();
        wr.start(
            "perhabs_canvas", // hardcode it
            web_options,
            Box::new(|cc| Box::new(app::Perhabs::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
