#![forbid(unsafe_code)]
//#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[macro_use]
extern crate tracing;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
  info!("Starting application");
  let app = intiface_desktop_egui::TemplateApp::default();
  let native_options = eframe::NativeOptions::default();
  eframe::run_native(Box::new(app), native_options);
}
