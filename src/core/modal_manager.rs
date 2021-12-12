use eframe::egui::Ui;
use std::sync::Arc;
use crate::core::AppCore;

pub trait ModalDialog: Sync + Send {
  fn render(&self, core: &mut AppCore, ui: &mut Ui);
}

#[derive(Default)]
pub struct ModalManager {
  modal: Option<Arc<Box<dyn ModalDialog>>>
}

impl ModalManager{
  pub fn set_modal_dialog(&mut self, dialog: impl ModalDialog + 'static) {
    self.modal = Some(Arc::new(Box::new(dialog)));
  }

  pub fn get_modal_dialog(&self) -> Option<Arc<Box<dyn ModalDialog>>> {
    self.modal.clone()
  }

  pub fn clear_modal_dialog(&mut self) {
    self.modal = None;
  }
}