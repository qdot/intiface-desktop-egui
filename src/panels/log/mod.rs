// Taken from tracing-egui by CAD97

mod archive;
mod layer;
mod panel;

#[cfg(feature = "smartstring")]
type SmartString = smartstring::SmartString<smartstring::LazyCompact>;
#[cfg(not(feature = "smartstring"))]
type SmartString = String;

pub use layer::EguiLayer;
pub use panel::LogPanel;

