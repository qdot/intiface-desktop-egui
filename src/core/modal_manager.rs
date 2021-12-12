use eframe::egui::Ui;
use std::sync::Arc;
use crate::core::AppCore;

pub trait ModalDialog: Sync + Send {
  fn render(&self, core: &mut AppCore, ui: &mut Ui);
}

struct OkModalDialog {
  label: String
}

impl OkModalDialog {
  pub fn new(label: &str) -> Self {
    Self {
      label: label.to_owned()
    }
  }
}

impl ModalDialog for OkModalDialog {
  fn render(&self, core: &mut AppCore, ui: &mut Ui) {
    ui.vertical(|ui| {
      ui.label(&self.label);
      ui.horizontal(|ui| {
        if ui.button("Ok").clicked() {
          core.modal_manager.clear_modal_dialog();
        }
      });
    });

  }
}

#[derive(Default)]
pub struct ModalManager {
  modal: Option<Arc<Box<dyn ModalDialog>>>
}

impl ModalManager{
  pub fn set_modal_dialog(&mut self, dialog: impl ModalDialog + 'static) {
    self.modal = Some(Arc::new(Box::new(dialog)));
  }

  pub fn set_ok_modal_dialog(&mut self, label: &str) {
    self.modal = Some(Arc::new(Box::new(OkModalDialog::new(label))));
  }

  pub fn get_modal_dialog(&self) -> Option<Arc<Box<dyn ModalDialog>>> {
    self.modal.clone()
  }

  pub fn clear_modal_dialog(&mut self) {
    self.modal = None;
  }
}