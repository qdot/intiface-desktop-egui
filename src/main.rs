#![forbid(unsafe_code)]
//#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate tracing;

use eframe::epi::IconData;
use std::io::Cursor;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
  info!("Starting application");
  let app_icon = include_bytes!("../icons/intiface-desktop-logo.ico");
  let app = intiface_desktop_egui::IntifaceDesktopApp::default();
  let mut native_options = eframe::NativeOptions::default();
  let reader = Cursor::new(app_icon);
  let icon_dir = ico::IconDir::read(reader).unwrap();
  // Decode the first entry into an image:
  let image = icon_dir.entries()[2].decode().unwrap();
  // You can get raw RGBA pixel data to pass to another image library:
  let rgba = image.rgba_data();
  native_options.icon_data = Some(IconData {
    rgba: rgba.to_vec(),
    width: image.width(),
    height: image.height()
  });
  eframe::run_native(Box::new(app), native_options);
}
